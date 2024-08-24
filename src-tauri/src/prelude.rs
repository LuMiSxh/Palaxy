use printpdf::image_crate;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Regex(#[from] regex::Error),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error(transparent)]
    Image(#[from] image::ImageError),
    #[error(transparent)]
    Epub(#[from] eyre::Report),
    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),
    #[error(transparent)]
    PrintPdf(#[from] printpdf::Error),
    #[error(transparent)]
    PrintPdfImage(#[from] image_crate::error::ImageError),
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

// App state
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct AppState {
    pub name: String,
    pub source: PathBuf,
    pub bundle_flag: BundleFlag,
    pub volume_sizes: Vec<usize>,
    pub data: Vec<Vec<PathBuf>>,
}

impl AppState {
    pub fn reset(&mut self) {
        self.source = PathBuf::default();
        self.bundle_flag = BundleFlag::default();
        self.volume_sizes = Vec::default();
        self.data = Vec::default();
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct CommandDefault {
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CommandGetData {
    pub message: Option<String>,
    pub data: Vec<Vec<PathBuf>>,
}

#[derive(Serialize, Deserialize)]
pub struct CommandBundle {
    pub message: Option<String>,
    pub total_chapters: usize,
    pub total_volumes: Option<usize>,
    pub chapter_sizes: Option<Vec<usize>>,
}

#[derive(Serialize, Deserialize)]
pub struct CommandAnalyze {
    pub message: Option<String>,
    pub negative: Vec<String>,
    pub positive: Vec<String>,
    pub suggest: Vec<String>,
    pub flag: BundleFlag,
}

// Types shared between frontend and tauri
#[derive(Serialize, Deserialize, Clone, Copy, Default)]
pub enum FileFormat {
    PDF,
    EPUB,
    #[default]
    CBZ,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Default)]
pub enum Direction {
    #[default]
    #[serde(rename = "Left to Right")]
    LTR,
    #[serde(rename = "Right to Left")]
    RTL,
}

#[derive(Serialize, Deserialize)]
pub struct AnalyzeResult {
    pub message: Option<String>,
    pub chapter_per_volume: Vec<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct ConvertResult {
    pub message: Option<String>,
}

// Workflow types
#[derive(Serialize, Deserialize, PartialEq, Clone, Default)]
pub enum BundleFlag {
    NAME,
    IMAGE,
    #[default]
    MANUAL,
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
