use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::PathBuf;
pub use crate::types::*;

// Error types
#[derive(Debug, thiserror::Error, Type)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    #[error(transparent)]
    Io(
        #[from]
        #[serde(skip)]
        std::io::Error,
    ),
    #[error(transparent)]
    Regex(
        #[from]
        #[serde(skip)]
        regex::Error,
    ),
    #[error(transparent)]
    Tauri(
        #[from]
        #[serde(skip)]
        tauri::Error,
    ),
    #[error(transparent)]
    Image(
        #[from]
        #[serde(skip)]
        image::ImageError,
    ),
    #[error(transparent)]
    Epub(
        #[from]
        #[serde(skip)]
        epub_builder::Error,
    ),
    #[error(transparent)]
    Zip(
        #[from]
        #[serde(skip)]
        zip::result::ZipError,
    ),
    #[error(transparent)]
    Reqwest(
        #[from]
        #[serde(skip)]
        reqwest::Error,
    ),
    #[error("The given path '{0}' is invalid: {1}")]
    InvalidPath(PathBuf, String),
    #[error("Asynchronous task failed: {0}")]
    AsyncTaskError(String),
    #[error("Unsupported: {0}")]
    Unsupported(String),
    #[error("Not found: {0}")]
    NotFound(String),
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

pub type EResult<T> = Result<T, Error>;

// App states
#[derive(Serialize, Deserialize, Clone, Default, Type, Debug)]
pub struct ConvState {
    pub name: String,
    pub source: PathBuf,
    pub target: PathBuf,
    pub bundle_flag: BundleFlag,
    pub direction: Direction,
    pub format: FileFormat,
    pub create_directory: bool,
    pub volume_sizes: Vec<usize>,
    pub data: Vec<Vec<PathBuf>>,
    pub edited_data: Option<Vec<Vec<PathBuf>>>,
}

impl ConvState {
    pub fn reset(&mut self) {
        *self = ConvState::default();
    }
}

pub struct AgentState {
    pub agents: Vec<Box<dyn Agent>>,
}

// Utils
pub fn get_file_info(image_path: &PathBuf) -> Result<(&'static str, &'static str), Error> {
    let path = image_path.extension().and_then(|e| e.to_str());

    match path {
        Some("jpg") | Some("jpeg") => Ok(("jpg", "image/jpeg")),
        Some("png") => Ok(("png", "image/png")),
        Some("webp") => Ok(("webp", "image/webp")),
        _ => Err(Error::Unsupported(format!("Image format {:#?}", path))),
    }
}
