//! Utility functions for file operations.

use crate::prelude::*;
use std::path::{Path, PathBuf};

/// Supported image file extensions (lowercase).
pub const SUPPORTED_IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "avif"];

/// Returns true if the file at the given path has a supported image extension.
#[inline]
pub fn is_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| SUPPORTED_IMAGE_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Determines file type and MIME type from a file path.
#[inline]
pub fn get_file_info(image_path: &PathBuf) -> EResult<(&'static str, &'static str)> {
    let path = image_path.extension().and_then(|e| e.to_str());

    match path {
        Some("jpg") | Some("jpeg") => Ok(("jpg", "image/jpeg")),
        Some("png") => Ok(("png", "image/png")),
        Some("webp") => Ok(("webp", "image/webp")),
        Some("avif") => Ok(("avif", "image/avif")),
        _ => Err(Error::Unsupported(format!("Image format {:#?}", path))),
    }
}
