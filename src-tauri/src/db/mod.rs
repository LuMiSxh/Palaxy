//! Database module for the Palaxy application.
//!
//! This module handles SQLite database connections, initialization, and transactions.
//! It provides a global connection pool that can be accessed throughout the application.

pub mod sync;

use crate::prelude::*;
use once_cell::sync::OnceCell;
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Sqlite, SqlitePool};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Manager};

/// Global database connection pool.
/// This is initialized once and can be accessed throughout the application.
static DB_POOL: OnceCell<SqlitePool> = OnceCell::new();

/// Gets the path to the database file.
///
/// Creates the application data directory if it doesn't exist.
///
/// # Arguments
///
/// * `handle` - The Tauri application handle.
///
/// # Returns
///
/// * `EResult<PathBuf>` - The path to the database file or an error.
pub fn get_db_path(handle: &AppHandle) -> EResult<PathBuf> {
    let app_dir = handle.path().app_data_dir()?;

    // Create the directory if it doesn't exist
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)
            .map_err(|e| Error::DatabaseError(format!("Failed to create app directory: {}", e)))?;
    }

    Ok(app_dir.join("palaxy.db"))
}

/// Initializes the database.
///
/// Creates the database if it doesn't exist, sets up a connection pool,
/// and runs migrations from the resource directory.
///
/// # Arguments
///
/// * `handle` - The Tauri application handle.
///
/// # Returns
///
/// * `EResult<SqlitePool>` - A connection pool to the database or an error.
pub async fn init_db(handle: &AppHandle) -> EResult<SqlitePool> {
    // Return existing pool if it exists
    if let Some(pool) = DB_POOL.get() {
        return Ok(pool.clone());
    }

    // Get the database path
    let db_path = get_db_path(handle)?;
    let db_url = format!("sqlite:{}", db_path.display());

    // Create the database if it doesn't exist
    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        Sqlite::create_database(&db_url)
            .await
            .map_err(|e| Error::DatabaseError(format!("Failed to create database: {}", e)))?;
    }

    // Create connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_url)
        .await
        .map_err(|e| Error::DatabaseError(format!("Failed to connect to database: {}", e)))?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| Error::DatabaseError(format!("Failed to run migrations: {}", e)))?;

    // Store pool in global static
    let _ = DB_POOL.set(pool.clone());

    Ok(pool)
}

/// Gets a connection from the pool.
///
/// Retrieves the global database connection pool if it exists.
///
/// # Returns
///
/// * `EResult<Option<SqlitePool>>` - The connection pool or None if it doesn't exist.
pub async fn get_pool() -> EResult<Option<SqlitePool>> {
    DB_POOL
        .get()
        .ok_or(Error::DatabaseError(
            "No database connection pool".to_string(),
        ))
        .map(|pool| Some(pool.clone()))
        .or_else(|_| Ok(None))
}

/// Helper function for database transactions.
///
/// Executes the given function within a database transaction,
/// automatically committing on success or rolling back on error.
///
/// # Arguments
///
/// * `f` - A function that takes a transaction reference and returns a future result.
///
/// # Returns
///
/// * `EResult<R>` - The result of the transaction or an error.
///
/// # Type Parameters
///
/// * `F` - A function type that takes a transaction and returns a boxed future.
/// * `R` - The return type of the function.
pub async fn with_transaction<'a, F, R>(f: F) -> EResult<R>
where
    F: FnOnce(&mut sqlx::Transaction<'_, Sqlite>) -> futures::future::BoxFuture<'a, EResult<R>>,
{
    let pool = get_pool().await?.ok_or(Error::DatabaseError(
        "No database connection pool".to_string(),
    ))?;
    let mut tx = pool.begin().await?;

    let result = f(&mut tx).await;

    match result {
        Ok(value) => {
            tx.commit().await?;
            Ok(value)
        }
        Err(err) => {
            let _ = tx.rollback().await;
            Err(err)
        }
    }
}
