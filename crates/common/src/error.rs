use specta::Type;
use std::path::PathBuf;

/// Application error types.
#[derive(Debug, thiserror::Error, Type)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    /// I/O error.
    #[error(transparent)]
    Io(
        #[from]
        #[serde(skip)]
        std::io::Error,
    ),
    /// Regular expression error.
    #[error(transparent)]
    Regex(
        #[from]
        #[serde(skip)]
        regex::Error,
    ),
    /// Tauri framework error.
    #[error(transparent)]
    Tauri(
        #[from]
        #[serde(skip)]
        tauri::Error,
    ),
    /// Image processing error.
    #[error(transparent)]
    Image(
        #[from]
        #[serde(skip)]
        image::ImageError,
    ),
    /// EPUB generation error.
    #[error(transparent)]
    Epub(
        #[from]
        #[serde(skip)]
        epub_builder::Error,
    ),
    /// ZIP operation error.
    #[error(transparent)]
    Zip(
        #[from]
        #[serde(skip)]
        zip::result::ZipError,
    ),
    /// Invalid file or directory path.
    #[error("The given path '{0}' is invalid: {1}")]
    InvalidPath(PathBuf, String),
    /// Asynchronous task failure.
    #[error("Asynchronous task failed: {0}")]
    AsyncTaskError(String),
    /// Unsupported operation or format.
    #[error("Unsupported: {0}")]
    Unsupported(String),
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

/// Result type alias using [`Error`].
pub type EResult<T> = Result<T, Error>;
