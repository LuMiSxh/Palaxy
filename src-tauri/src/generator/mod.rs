//! Generator module provides traits and implementations for various file format generators.
//!
//! This module contains the common interface for document generators and specific
//! implementations for different file formats.

use crate::prelude::*;
use std::path::PathBuf;

pub mod cbz;
pub mod epub;

/// Common interface for all file generators.
///
/// The `Generator` trait defines a consistent API for document generators
/// that can create different file formats (like CBZ, EPUB) from source images.
/// Implementations handle the specifics of each file format.
pub trait Generator {
    /// Creates a new generator instance.
    ///
    /// # Parameters
    /// * `output_path` - Directory where the generated file will be saved
    /// * `filename` - Name of the output file (without extension)
    ///
    /// # Returns
    /// * `EResult<Self>` - A new generator instance or an error if creation fails
    fn new(output_path: &str, filename: &str) -> EResult<Self>
    where
        Self: Sized;

    /// Adds a page to the generated document.
    ///
    /// # Parameters
    /// * `image_path` - Path to the image file to add as a page
    ///
    /// # Returns
    /// * `EResult<&mut Self>` - Self reference for method chaining, or an error if failed
    fn add_page(&mut self, image_path: &PathBuf) -> EResult<&mut Self>
    where
        Self: Sized;

    /// Sets metadata for the generated document.
    ///
    /// # Parameters
    /// * `title` - Title of the document
    /// * `volume` - Volume number
    ///
    /// # Returns
    /// * `EResult<&mut Self>` - Self reference for method chaining, or an error if failed
    fn set_metadata(&mut self, title: &str, volume: usize) -> EResult<&mut Self>
    where
        Self: Sized;

    /// Saves the generated document to disk.
    ///
    /// Finalizes the document and writes it to the specified output location.
    ///
    /// # Returns
    /// * `EResult<()>` - Success indicator or an error if saving fails
    fn save(self) -> EResult<()>;
}
