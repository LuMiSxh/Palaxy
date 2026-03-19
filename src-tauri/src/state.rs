use std::path::PathBuf;
use std::sync::Arc;

use common::{BundleFlag, Direction, FileFormat, ImageOutputFormat};
use serde::{Deserialize, Serialize};
use specta::Type;
use tempfile::TempDir;

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
    /// When true, merge all chapters into a single output volume.
    pub flatten: bool,
    /// Whether the input was detected as a flat structure (no chapter subdirectories).
    #[serde(skip)]
    pub is_flat: bool,
    /// Non-image files found during scanning (for warning display).
    #[serde(skip)]
    pub skipped_files: Vec<PathBuf>,
    /// Temp directory handle for ZIP extraction — not serialized.
    /// Kept alive to prevent cleanup of extracted files.
    #[serde(skip)]
    #[specta(skip)]
    pub temp_dir: Option<Arc<TempDir>>,
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
