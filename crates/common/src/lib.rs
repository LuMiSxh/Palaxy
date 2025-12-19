//! Common types, errors, and utilities for Palaxy.
//!
//! # Usage
//!
//! Import the prelude for most common items:
//!
//! ```rust
//! use common::prelude::*;
//! ```
//!
//! Or use short paths for specific imports:
//!
//! ```rust
//! use common::{Error, EResult, BundleFlag, FileFormat};
//! ```

// Module declarations
pub mod error;
pub mod image_utils;
pub mod macros;
pub mod resource;
pub mod types;
pub mod utils;

// Re-export commonly used items at crate root for short paths
pub use error::{EResult, Error};
pub use image_utils::{is_grayscale, is_grayscale_with_threshold, GrayscaleStrategy};
pub use resource::ResourceBudget;
pub use types::{
    AnalyzeResponse, BaseResponse, BundleFlag, BundleResponse, CommAnalyzeMeta, CommBundle,
    ConvStateKey, Direction, FileFormat, ImageOutputFormat,
};
pub use utils::get_file_info;

/// Prelude module for convenient imports.
pub mod prelude {
    // Error handling
    pub use crate::error::{EResult, Error};

    // Image utilities
    pub use crate::image_utils::{is_grayscale, is_grayscale_with_threshold, GrayscaleStrategy};

    // Resource management
    pub use crate::resource::ResourceBudget;

    // All types
    pub use crate::types::{
        AnalyzeResponse, BaseResponse, BundleFlag, BundleResponse, CommAnalyzeMeta, CommBundle,
        ConvStateKey, Direction, FileFormat, ImageOutputFormat,
    };

    // Utilities
    pub use crate::utils::get_file_info;

    // Re-export commonly used external types
    pub use serde::{Deserialize, Serialize};
    pub use specta::Type;
    pub use std::path::PathBuf;
}
