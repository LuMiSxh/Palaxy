//! Document generator trait and implementations.

use common::prelude::*;

pub mod cbz;
pub mod epub;

/// Generator interface for all output formats.
pub trait Generator {
    /// Creates new generator instance.
    fn new(output_path: &str, filename: &str) -> EResult<Self>
    where
        Self: Sized;

    /// Adds page from in-memory image data.
    fn add_page_from_memory(&mut self, data: &[u8], extension: &str) -> EResult<&mut Self>
    where
        Self: Sized;

    /// Sets metadata for the document.
    fn set_metadata(&mut self, title: &str, volume: usize) -> EResult<&mut Self>
    where
        Self: Sized;

    /// Finalizes and saves the document to disk.
    fn save(self) -> EResult<()>;
}
