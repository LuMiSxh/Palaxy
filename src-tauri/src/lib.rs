//! Main application initialization module.
//!
//! This module is responsible for setting up and running the Tauri application.
//! It handles command registration, TypeScript bindings export, plugin setup,
//! and application state initialization.

use crate::commands::converter::{
    conv_analyze, conv_bundle, conv_convert, conv_state_get, conv_state_reset, conv_state_set,
};
use crate::commands::management::{
    mgmt_get_logs_path, mgmt_sync_start, mgmt_sync_status, mgmt_sync_stop,
};
use crate::db::init_db;
use crate::db::sync::SyncManager;
use specta_typescript::Typescript;
use tauri::async_runtime::block_on;
use tauri::{Builder, Manager};
use tauri_specta::{collect_commands, Builder as SpectaBuilder};
use tokio::sync::Mutex;

mod collector;
mod commands;
mod generator;
mod prelude;
mod types;
#[macro_use]
mod macros;
mod db;

/// Initializes and runs the Tauri application.
///
/// This function performs the following tasks:
/// 1. Register commands for frontend-backend communication
/// 2. Export TypeScript bindings for development build
/// 3. Configure the application with necessary plugins
/// 4. Set up application state, including database initialization
/// 5. Start the Tauri application with the configured settings
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Register commands for the frontend to call
    let spectra_builder = SpectaBuilder::<tauri::Wry>::new().commands(collect_commands![
        conv_state_set,
        conv_state_get,
        conv_state_reset,
        conv_analyze,
        conv_bundle,
        conv_convert,
        mgmt_sync_start,
        mgmt_sync_stop,
        mgmt_sync_status,
        mgmt_get_logs_path
    ]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    {
        // Configure TypeScript bindings export for development
        let mut ts = Typescript::default();
        ts = ts.bigint(specta_typescript::BigIntExportBehavior::Number);

        spectra_builder
            .export(ts, "../src/types/bindings.ts")
            .expect("Failed to export typescript bindings");
    }

    Builder::default()
        // Ensure only one instance of the app runs at a time
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }))
        // Configure application logging
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("logs".to_string()),
                    },
                ))
                .level(log::LevelFilter::Info)
                .format(|out, message, record| {
                    let time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
                    out.finish(format_args!(
                        "[{} {} {}] [{}:{}] {}",
                        time,
                        record.level(),
                        record.target(),
                        record.file().unwrap_or("unknown"),
                        record.line().unwrap_or(0),
                        message
                    ))
                })
                .build(),
        )
        // Add OS and dialog functionality plugins
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        // Application setup callback
        .setup(|app| {
            // Initialize application state
            app.manage(Mutex::new(prelude::ConvState::default()));

            let handle = app.handle();

            // Configure and start database synchronization
            let manager = SyncManager::new(60); // Sync every 60 minutes
            manager.start();
            app.manage(manager);

            // Initialize the database asynchronously
            block_on(async move {
                init_db(handle)
                    .await
                    .expect("Failed to initialize database");
            });

            Ok(())
        })
        .invoke_handler(spectra_builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
