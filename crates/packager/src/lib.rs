//! Image packaging and format generation for comic/manga files.
//!
//! # Usage
//!
//! Import the prelude for most common items:
//!
//! ```rust
//! use packager::prelude::*;
//! ```
//!
//! Or use short paths for specific imports:
//!
//! ```rust
//! use packager::{Generator, process_images_to_memory};
//! ```

// Module declarations
pub mod generator;
pub mod image;

// Re-export commonly used items at crate root for short paths
pub use generator::{cbz::Cbz, epub::EPub, Generator};
pub use image::{process_images_to_memory, ProcessedPage};

/// Prelude module for convenient imports.
pub mod prelude {
    // Generator types and trait
    pub use crate::generator::{cbz::Cbz, epub::EPub, Generator};

    // Image processing
    pub use crate::image::{process_images_to_memory, ProcessedPage};

    // Re-export common types from common crate
    pub use common::prelude::*;
}
