use crate::prelude::*;
use std::path::PathBuf;
use async_trait::async_trait;

pub mod cbz;
pub mod epub;

/// Common interface for all file generators
#[async_trait]
pub trait Generator {
    /// Creates a new generator instance
    fn new(output_path: &str, filename: &str) -> EResult<Self> where Self: Sized;

    /// Adds a page to the generated document
    async fn add_page(&mut self, image_path: &PathBuf) -> EResult<&mut Self> where Self: Sized;

    /// Sets metadata for the generated document
    async fn set_metadata(&mut self, title: &str, volume: usize) -> EResult<&mut Self> where Self: Sized;

    /// Saves the generated document
    async fn save(self) -> EResult<()>;
}
