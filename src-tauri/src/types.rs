use serde::{Deserialize, Serialize};
use specta::Type;

// --- Enums ---

#[derive(Serialize, Deserialize, Clone, Copy, Default, Type)]
pub enum BundleFlag {
    #[serde(rename = "NAME")]
    Name,
    #[serde(rename = "IMAGE")]
    Image,
    #[default]
    #[serde(rename = "MANUAL")]
    Manual,
}

#[derive(Serialize, Deserialize, Clone, Copy, Default, Type)]
pub enum FileFormat {
    #[serde(rename = "PDF")]
    Pdf,
    #[serde(rename = "EPUB")]
    Epub,
    #[default]
    #[serde(rename = "CBZ")]
    Cbz,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Default, Type)]
pub enum Direction {
    #[default]
    #[serde(rename = "Left to Right")]
    Ltr,
    #[serde(rename = "Right to Left")]
    Rtl,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Default, Type)]
pub enum StatusFlag {
    Experimental,
    Deprecated,
    Broken,
    #[default]
    Stable,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Type)]
pub enum TagType {
    Language(String),
    Status(StatusFlag),
    Genre(String),
    Other(String),
}

// --- Structs ---
#[derive(Serialize, Deserialize, Debug, Default, Type)]
pub struct Element {
    id: String,
    title: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Type)]
pub struct AgentMeta {
    name: String,
    url: String,
    icon: Option<String>,
    tags: Vec<TagType>,
}

// --- Responses ---

#[derive(Serialize, Deserialize, Default, Type)]
pub struct BaseResponse<T = ()> {
    pub duration: u64,
    pub comment: Option<String>,
    pub data: Option<T>,
}

pub type CommGetData = BaseResponse<Vec<Vec<String>>>; // <- was: CommandGetData

#[derive(Serialize, Deserialize, Default, Type)]
pub struct BundleResponse {
    pub total_chapters: usize,
    pub total_volumes: Option<usize>,
    pub chapter_sizes: Option<Vec<usize>>,
}
pub type CommBundle = BaseResponse<BundleResponse>; // <- was: CommandBundle

#[derive(Serialize, Deserialize, Default, Type)]
pub struct AnalyzeResponse {
    pub negative: Vec<String>,
    pub positive: Vec<String>,
    pub suggest: Vec<String>,
    pub flag: BundleFlag,
}
pub type CommAnalyzeMeta = BaseResponse<AnalyzeResponse>; // <- was: CommandAnalyze

pub type CommListAgents = BaseResponse<Vec<AgentMeta>>; // <- was: CommandListAgents

pub type CommAnalyzeCPV = BaseResponse<Vec<usize>>; // <- was: AnalyzeResult
