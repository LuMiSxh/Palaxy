pub use crate::types::*;
/// Prelude module containing common types, error handling, and utilities
/// for the Tauri application.
///
/// This module provides essentials used throughout the application including:
/// - Error types and handling
/// - Application state management
/// - Utility functions for file operations
use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::PathBuf;

/// Error types that can occur during application execution
///
/// Provides a unified error handling system that wraps both standard library
/// and third-party errors, as well as application-specific errors.
#[derive(Debug, thiserror::Error, Type)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    /// I/O errors from the standard library
    #[error(transparent)]
    Io(
        #[from]
        #[serde(skip)]
        std::io::Error,
    ),
    /// Regular expression parsing errors
    #[error(transparent)]
    Regex(
        #[from]
        #[serde(skip)]
        regex::Error,
    ),
    /// Errors from the Tauri framework
    #[error(transparent)]
    Tauri(
        #[from]
        #[serde(skip)]
        tauri::Error,
    ),
    /// Image processing errors
    #[error(transparent)]
    Image(
        #[from]
        #[serde(skip)]
        image::ImageError,
    ),
    /// EPUB generation errors
    #[error(transparent)]
    Epub(
        #[from]
        #[serde(skip)]
        epub_builder::Error,
    ),
    /// ZIP file operation errors
    #[error(transparent)]
    Zip(
        #[from]
        #[serde(skip)]
        zip::result::ZipError,
    ),
    /// HTTP request errors
    #[error(transparent)]
    Reqwest(
        #[from]
        #[serde(skip)]
        reqwest::Error,
    ),
    /// Error for invalid file or directory paths
    #[error("The given path '{0}' is invalid: {1}")]
    InvalidPath(PathBuf, String),
    /// Error for failed asynchronous tasks
    #[error("Asynchronous task failed: {0}")]
    AsyncTaskError(String),
    /// Error for unsupported operations or formats
    #[error("Unsupported: {0}")]
    Unsupported(String),
    /// Error for resources that couldn't be found
    #[error("Not found: {0}")]
    NotFound(String),
    /// Generic database error
    #[error("Database error: {0}")]
    DatabaseError(String),
    /// SQLx database errors
    #[error(transparent)]
    SqlxError(
        #[from]
        #[serde(skip)]
        sqlx::Error,
    ),
    /// SQLx migration errors
    #[error(transparent)]
    SqlxMigrationError(
        #[from]
        #[serde(skip)]
        sqlx::migrate::MigrateError,
    ),
}

/// Implementation for serializing Error types to strings
///
/// This allows errors to be safely transmitted to the frontend
/// by converting them to human-readable messages.
impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

/// Type alias for Results that use the application's Error type
pub type EResult<T> = Result<T, Error>;

/// Represents the state of a file conversion operation
///
/// Contains all necessary information about a conversion task including
/// source and destination paths, format settings, and file organization.
#[derive(Serialize, Deserialize, Clone, Default, Type, Debug)]
pub struct ConvState {
    /// Name of the conversion task
    pub name: String,
    /// Source directory or file path
    pub source: PathBuf,
    /// Target directory for output files
    pub target: PathBuf,
    /// Method used for bundling files
    pub bundle_flag: BundleFlag,
    /// Reading direction (LTR or RTL)
    pub direction: Direction,
    /// Output file format (EPUB or CBZ)
    pub format: FileFormat,
    /// Whether to create a new directory for output
    pub create_directory: bool,
    /// Sizes for volume splitting
    pub volume_sizes: Vec<usize>,
    /// Collection of file paths organized for processing
    pub data: Vec<Vec<PathBuf>>,
    /// Optional edited version of the data collection
    pub edited_data: Option<Vec<Vec<PathBuf>>>,
}

impl ConvState {
    /// Resets the conversion state to default values
    ///
    /// This clears all fields by replacing the current instance with a new default instance.
    pub fn reset(&mut self) {
        *self = ConvState::default();
    }
}

/// Utility functions for file operations

/// Determines file type and MIME type from a file path
///
/// # Arguments
///
/// * `image_path` - Path to the file to analyze
///
/// # Returns
///
/// * `Ok((&str, &str))` - A tuple containing (file extension, MIME type)
/// * `Err(Error)` - An error if the file format is unsupported
///
/// # Supported formats
///
/// - JPEG/JPG: image/jpeg
/// - PNG: image/png
/// - WebP: image/webp
pub fn get_file_info(image_path: &PathBuf) -> Result<(&'static str, &'static str), Error> {
    let path = image_path.extension().and_then(|e| e.to_str());

    match path {
        Some("jpg") | Some("jpeg") => Ok(("jpg", "image/jpeg")),
        Some("png") => Ok(("png", "image/png")),
        Some("webp") => Ok(("webp", "image/webp")),
        _ => Err(Error::Unsupported(format!("Image format {:#?}", path))),
    }
}

/// Status information about the synchronization service
#[derive(Serialize, Deserialize, Type, Debug, Clone)]
pub struct SyncStatus {
    /// Seconds until the next sync operation
    pub seconds_until_next_sync: Option<u64>,
    /// Minutes until the next sync operation
    pub minutes_until_next_sync: Option<u64>,
}

/// Path information for application log files
#[derive(Serialize, Deserialize, Type, Debug, Clone)]
pub struct LogPath {
    /// Directory where log files are stored
    pub directory: String,
    /// Full path to the main log file
    pub file: String,
}

