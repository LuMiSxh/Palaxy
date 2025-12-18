//! Application prelude module.

// Re-export common types and errors from the common crate
pub use common::prelude::*;

use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::PathBuf;

/// Conversion state for the application.
#[derive(Serialize, Deserialize, Clone, Default, Type, Debug)]
#[serde(default)]
pub struct ConvState {
    /// Name of the conversion task.
    pub name: String,
    /// Source directory or file path.
    pub source: PathBuf,
    /// Target directory for output files.
    pub target: PathBuf,
    /// Method used for bundling files.
    pub bundle_flag: BundleFlag,
    /// Reading direction (LTR or RTL).
    pub direction: Direction,
    /// Output file format (EPUB or CBZ).
    pub format: FileFormat,
    /// Whether to create a new directory for output.
    pub create_directory: bool,
    /// Whether to convert images to WebP format (deprecated, use image_format instead).
    pub convert_to_webp: bool,
    /// Image output format for conversion.
    pub image_format: ImageOutputFormat,
    /// Sizes for volume splitting.
    pub volume_sizes: Vec<usize>,
    /// Collection of file paths organized for processing.
    pub data: Vec<Vec<PathBuf>>,
    /// Optional edited version of the data collection.
    pub edited_data: Option<Vec<Vec<PathBuf>>>,
    /// Whether to hide the volume number when there's only one volume.
    pub hide_single_volume_number: bool,
    /// Custom separator string between project name and volume number.
    #[serde(default = "default_volume_separator")]
    pub volume_separator: String,
}

/// Default value for volume separator.
fn default_volume_separator() -> String {
    " - ".to_string()
}

impl ConvState {
    /// Resets the conversion state to default values.
    pub fn reset(&mut self) {
        *self = ConvState::default();
    }
}

/// Path information for application log files.
#[derive(Serialize, Deserialize, Type, Debug, Clone)]
pub struct LogPath {
    /// Directory where log files are stored.
    pub directory: String,
    /// Full path to the main log file.
    pub file: String,
}
