//! Utility functions for file operations.

use crate::prelude::*;
use std::path::PathBuf;

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
