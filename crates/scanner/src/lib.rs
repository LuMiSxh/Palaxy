//! Image collection and analysis for comic/manga processing.
//!
//! # Usage
//!
//! Import the prelude for most common items:
//!
//! ```rust
//! use scanner::prelude::*;
//! ```
//!
//! Or use short paths for specific imports:
//!
//! ```rust
//! use scanner::{Collector, determine_volume_start_chapters};
//! ```

// Module declarations
pub mod analyzer;
pub mod collector;
pub mod constants;
pub mod sorting;

// Re-export commonly used items at crate root for short paths
pub use analyzer::{calculate_volume_sizes, determine_volume_start_chapters};
pub use collector::Collector;
pub use constants::MAX_CONCURRENT_DIRS;
pub use sorting::{sort_by_name_volume_chapter, sort_by_stem_number, sort_name_by_number};

/// Prelude module for convenient imports.
pub mod prelude {
    // Collector
    pub use crate::collector::Collector;

    // Analyzer functions
    pub use crate::analyzer::{calculate_volume_sizes, determine_volume_start_chapters};

    // Sorting functions
    pub use crate::sorting::{
        sort_by_name_volume_chapter, sort_by_stem_number, sort_name_by_number,
    };

    // Constants
    pub use crate::constants::MAX_CONCURRENT_DIRS;

    // Re-export common types
    pub use common::prelude::*;
}
