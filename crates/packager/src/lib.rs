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
pub mod pipeline;

// Re-export commonly used items at crate root for short paths
pub use generator::{cbz::Cbz, epub::EPub, Generator};
pub use image::{
    convert_to_avif, convert_to_webp, process_images_to_memory,
    process_images_to_memory_with_options, ProcessedPage,
};
pub use pipeline::{process_and_write_streaming, process_and_write_streaming_with_options};

/// Prelude module for convenient imports.
pub mod prelude {
    // Generator types and trait
    pub use crate::generator::{cbz::Cbz, epub::EPub, Generator};

    // Image processing
    pub use crate::image::{convert_to_avif, convert_to_webp, process_images_to_memory, ProcessedPage};

    // Re-export common types from common crate
    pub use common::prelude::*;
}
