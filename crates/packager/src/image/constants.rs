//! Tunable constants for image processing and encoding.

// --- AVIF Encoding ---

/// Base quality for small images (< 800x600).
pub const AVIF_QUALITY_SMALL: f32 = 82.0;

/// Base quality for medium images (< 3MP).
pub const AVIF_QUALITY_MEDIUM: f32 = 80.0;

/// Base quality for large images (≥ 3MP).
pub const AVIF_QUALITY_LARGE: f32 = 78.0;

/// Speed preset for small images (0-10, higher = faster).
pub const AVIF_SPEED_SMALL: u8 = 8;

/// Speed preset for medium images.
pub const AVIF_SPEED_MEDIUM: u8 = 7;

/// Speed preset for large images.
pub const AVIF_SPEED_LARGE: u8 = 6;

/// Quality reduction for grayscale images.
pub const AVIF_GRAYSCALE_QUALITY_REDUCTION: f32 = 2.0;

/// Pixel count threshold for small images (~800x600).
pub const AVIF_SMALL_IMAGE_THRESHOLD: u64 = 480_000;

/// Pixel count threshold for medium images (~3MP).
pub const AVIF_MEDIUM_IMAGE_THRESHOLD: u64 = 3_000_000;

// --- WebP Encoding ---

/// Base quality for small images (0-100 scale).
pub const WEBP_QUALITY_SMALL: f32 = 85.0;

/// Base quality for medium images (0-100 scale).
pub const WEBP_QUALITY_MEDIUM: f32 = 82.0;

/// Base quality for large images (0-100 scale).
pub const WEBP_QUALITY_LARGE: f32 = 80.0;

/// Compression method for small images (0-6, higher = slower).
pub const WEBP_METHOD_SMALL: u8 = 4;

/// Compression method for medium images (0-6, higher = slower).
pub const WEBP_METHOD_MEDIUM: u8 = 5;

/// Compression method for large images (0-6, higher = slower).
pub const WEBP_METHOD_LARGE: u8 = 6;

/// Quality reduction for grayscale images.
pub const WEBP_GRAYSCALE_QUALITY_REDUCTION: f32 = 3.0;

/// Pixel count threshold for small images (~800x600).
pub const WEBP_SMALL_IMAGE_THRESHOLD: u64 = 480_000;

/// Pixel count threshold for medium images (~3MP).
pub const WEBP_MEDIUM_IMAGE_THRESHOLD: u64 = 3_000_000;

// --- Performance ---

/// Minimum chunk size for parallel processing.
pub const MIN_PARALLEL_CHUNK_SIZE: usize = 1;
