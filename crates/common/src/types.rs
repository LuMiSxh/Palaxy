use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::PathBuf;

// --- Enums ---

/// File bundling method.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Default, Type)]
pub enum BundleFlag {
    #[serde(rename = "NAME")]
    Name,
    #[serde(rename = "IMAGE")]
    Image,
    #[default]
    #[serde(rename = "MANUAL")]
    Manual,
}

/// Output file format.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Default, Type)]
pub enum FileFormat {
    #[serde(rename = "EPUB")]
    Epub,
    #[default]
    #[serde(rename = "CBZ")]
    Cbz,
}

/// Reading direction for ePub content.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Default, Type)]
pub enum Direction {
    #[default]
    #[serde(rename = "Left to Right")]
    Ltr,
    #[serde(rename = "Right to Left")]
    Rtl,
}

/// Image output format for conversion.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Default, Type)]
pub enum ImageOutputFormat {
    #[default]
    #[serde(rename = "None")]
    None,
    #[serde(rename = "WebP")]
    WebP,
    #[serde(rename = "AVIF")]
    Avif,
}

/// Conversion state property keys.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Type)]
pub enum ConvStateKey {
    Name(String),
    Source(PathBuf),
    Target(PathBuf),
    BundleFlag(BundleFlag),
    Direction(Direction),
    Format(FileFormat),
    CreateDirectory(bool),
    ConvertToWebp(bool),
    ImageFormat(ImageOutputFormat),
    VolumeSizes(Vec<usize>),
    Data(Vec<Vec<PathBuf>>),
    EditedData(Option<Vec<Vec<PathBuf>>>),
    HideSingleVolumeNumber(bool),
    VolumeSeparator(String),
}

// --- Responses ---

/// Base response structure with optional payload.
#[derive(Serialize, Deserialize, Default, Type)]
pub struct BaseResponse<T = ()> {
    pub duration: f64,
    pub comment: Option<String>,
    pub payload: Option<T>,
}

impl BaseResponse<()> {
    /// Creates a response with only duration set.
    pub fn default_duration(duration: f64) -> Self {
        Self {
            duration,
            comment: None,
            payload: None,
        }
    }
}

/// Bundling operation results.
#[derive(Serialize, Deserialize, Default, Type)]
pub struct BundleResponse {
    pub total_chapters: usize,
    pub total_volumes: Option<usize>,
    pub chapter_sizes: Option<Vec<usize>>,
}

/// Bundle operation response type.
pub type CommBundle = BaseResponse<BundleResponse>;

/// Analysis operation results.
#[derive(Serialize, Deserialize, Default, Type)]
pub struct AnalyzeResponse {
    pub negative: Vec<String>,
    pub positive: Vec<String>,
    pub warning: Vec<String>,
    pub flag: BundleFlag,
}

/// Analysis operation response type.
pub type CommAnalyzeMeta = BaseResponse<AnalyzeResponse>;
