//! Converter command module with modular organization
//!
//! This module provides the main conversion functionality, split into logical submodules:
//! - `state`: State management (get, set, reset)
//! - `analyze`: Source directory analysis
//! - `bundle`: Chapter bundling logic
//! - `convert`: Main conversion orchestration with parallel processing
//! - `image`: Image format conversion with optimized encoders
//! - `events`: Progress event definitions and emission helpers

pub mod analyze;
pub mod bundle;
pub mod convert;
pub mod events;
pub mod image;
pub mod state;

// Re-export public command functions
pub use analyze::conv_analyze;
pub use bundle::conv_bundle;
pub use convert::conv_convert;
pub use state::{conv_state_get, conv_state_reset, conv_state_set};

// Re-export events for registration
pub use events::{
    ConversionCompleteEvent, ConversionStartEvent, ImageProgressEvent, StatusMessageEvent,
    VolumeCompleteEvent, VolumeStartEvent,
};
