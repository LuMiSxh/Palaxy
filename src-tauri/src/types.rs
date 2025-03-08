use crate::prelude::EResult;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::PathBuf;
// --- Enums ---

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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Default, Type)]
pub enum FileFormat {
    #[serde(rename = "PDF")]
    Pdf,
    #[serde(rename = "EPUB")]
    Epub,
    #[default]
    #[serde(rename = "CBZ")]
    Cbz,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Default, Type)]
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
    #[default]
    Stable,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Type)]
pub enum TagType {
    Language(String),
    Status(StatusFlag),
    Other(String),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Type)]
pub enum ConvStateKey {
    Name(String),
    Source(PathBuf),
    BundleFlag(BundleFlag),
    Direction(Direction),
    Format(FileFormat),
    CreateDirectory(bool),
    VolumeSizes(Vec<usize>),
    Data(Vec<Vec<PathBuf>>),
}

// --- Structs ---
#[derive(Serialize, Deserialize, Debug, Default, Type)]
pub struct Element {
    pub id: String,
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Type)]
pub struct AgentMeta {
    pub name: String,
    pub url: String,
    pub icon: Option<String>,
    pub tags: Vec<TagType>,
}

pub type Url = String;

#[async_trait]
pub trait Agent: Send + Sync {
    fn new(client: Client) -> Self
    where
        Self: Sized;

    fn representation(&self) -> AgentMeta;

    async fn get_mangas(&self) -> EResult<Vec<Element>>;

    async fn get_chapters(&self, manga: Element) -> EResult<Vec<Element>>;

    async fn get_pages(&self, chapter: Element) -> EResult<Vec<Url>>;
}

// --- Responses ---

#[derive(Serialize, Deserialize, Default, Type)]
pub struct BaseResponse<T = ()> {
    pub duration: u64,
    pub comment: Option<String>,
    pub payload: Option<T>,
}

impl BaseResponse<()> {
    pub fn default_duration(duration: u64) -> Self {
        Self {
            duration,
            comment: None,
            payload: None,
        }
    }
}

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
