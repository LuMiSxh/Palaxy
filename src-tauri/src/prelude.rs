use std::path::PathBuf;
use serde::{Deserialize, Serialize};

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

// Types shared between frontend and tauri
#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum FileFormat {
    PDF,
    EPUB,
    CBZ,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum Direction {
    #[serde(rename = "Left to Right")]
    LTR,
    #[serde(rename = "Right to Left")]
    RTL,
}

#[derive(Serialize, Deserialize)]
pub struct AnalyzeResult {
    pub message: String,
    pub chapter_per_volume: Vec<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct ConvertResult {
    pub message: String
}

// Utils
pub fn get_file_info(image_path: &PathBuf) -> Result<(&'static str, &'static str), Error> {
    let path = image_path.extension().and_then(|e| e.to_str());

    match path {
        Some("jpg") | Some("jpeg") => Ok(("jpg", "image/jpeg")),
        Some("png") => Ok(("png", "image/png")),
        Some("webp") => Ok(("webp", "image/webp")),
        _ => return Err(Error::Unsupported(format!("Image format {:#?}", path))),
    }
}