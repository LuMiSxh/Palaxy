use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::PathBuf;
// --- Enums ---

/// Represents the method used for bundling files
///
/// * `Name` - Bundle by name
/// * `Image` - Bundle by image
/// * `Manual` - Manual bundling (default)
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

/// Supported file formats for conversion
///
/// * `Epub` - Electronic Publication format
/// * `Cbz` - Comic Book ZIP format (default)
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Default, Type)]
pub enum FileFormat {
    #[serde(rename = "EPUB")]
    Epub,
    #[default]
    #[serde(rename = "CBZ")]
    Cbz,
}

/// Reading direction for content in an ePub file
///
/// * `Ltr` - Left to Right (default)
/// * `Rtl` - Right to Left
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Default, Type)]
pub enum Direction {
    #[default]
    #[serde(rename = "Left to Right")]
    Ltr,
    #[serde(rename = "Right to Left")]
    Rtl,
}

/// Indicates the stability status of a feature
///
/// * `Experimental` - Feature is under testing
/// * `Deprecated` - Feature will be removed in future versions
/// * `Stable` - Feature is ready for production use (default)
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Default, Type)]
pub enum StatusFlag {
    Experimental,
    Deprecated,
    #[default]
    Stable,
}

/// Represents different types of tags that can be applied
///
/// * `Language` - Language-specific tag with associated string
/// * `Status` - Status indicator using StatusFlag
/// * `Other` - Custom tag with associated string
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Type)]
pub enum TagType {
    Language(String),
    Status(StatusFlag),
    Other(String),
}

/// Keys for the conversion state data
///
/// Represents various properties that can be set during the conversion process
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Type)]
pub enum ConvStateKey {
    Name(String),
    Source(PathBuf),
    Target(PathBuf),
    BundleFlag(BundleFlag),
    Direction(Direction),
    Format(FileFormat),
    CreateDirectory(bool),
    VolumeSizes(Vec<usize>),
    Data(Vec<Vec<PathBuf>>),
    EditedData(Option<Vec<Vec<PathBuf>>>),
}

// --- Responses ---

/// Base response structure that can contain optional payload data
///
/// * `duration` - Operation execution time in seconds
/// * `comment` - Optional comment or message related to the operation
/// * `payload` - Optional data payload of generic type T
#[derive(Serialize, Deserialize, Default, Type)]
pub struct BaseResponse<T = ()> {
    pub duration: f64,
    pub comment: Option<String>,
    pub payload: Option<T>,
}

impl BaseResponse<()> {
    /// Creates a new BaseResponse with only duration set
    ///
    /// # Arguments
    ///
    /// * `duration` - The operation execution time in seconds
    ///
    /// # Returns
    ///
    /// A new BaseResponse with duration set and other fields defaulted
    pub fn default_duration(duration: f64) -> Self {
        Self {
            duration,
            comment: None,
            payload: None,
        }
    }
}

/// Response structure containing bundling operation results
///
/// * `total_chapters` - Total number of chapters processed
/// * `total_volumes` - Optional total number of volumes created
/// * `chapter_sizes` - Optional list of chapter sizes in bytes
#[derive(Serialize, Deserialize, Default, Type)]
pub struct BundleResponse {
    pub total_chapters: usize,
    pub total_volumes: Option<usize>,
    pub chapter_sizes: Option<Vec<usize>>,
}

/// Type alias for a BaseResponse containing BundleResponse data
pub type CommBundle = BaseResponse<BundleResponse>;

/// Response structure containing analysis results
///
/// * `negative` - List of negative findings
/// * `positive` - List of positive findings
/// * `warning` - List of warnings
/// * `flag` - Recommended bundle flag based on analysis
#[derive(Serialize, Deserialize, Default, Type)]
pub struct AnalyzeResponse {
    pub negative: Vec<String>,
    pub positive: Vec<String>,
    pub warning: Vec<String>,
    pub flag: BundleFlag,
}

/// Type alias for a BaseResponse containing AnalyzeResponse data
pub type CommAnalyzeMeta = BaseResponse<AnalyzeResponse>;
