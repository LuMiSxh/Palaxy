//! Tunable constants for image processing and encoding.
//!
//! These constants are optimized for BOTH speed AND file size, based on
//! encoder performance characteristics and typical manga content.

// ============================================================================
// AVIF ENCODING
// ============================================================================

/// Base quality for tiny images (< 300k pixels ~500x600).
/// These are thumbnails or very low-res - use higher quality to avoid artifacts.
pub const AVIF_QUALITY_TINY: f32 = 72.0;

/// Base quality for small images (< 1M pixels ~800x1200).
/// Older scans or compressed sources - maintain detail.
pub const AVIF_QUALITY_SMALL: f32 = 70.0;

/// Base quality for medium images (< 3M pixels ~1400x2100).
/// **Most manga pages fall here** - optimized for balance.
pub const AVIF_QUALITY_MEDIUM: f32 = 68.0;

/// Base quality for large images (< 6M pixels ~2000x3000).
/// High-res scans - can afford slightly lower quality.
pub const AVIF_QUALITY_LARGE: f32 = 65.0;

/// Base quality for huge images (≥ 6M pixels, double spreads).
/// Very large images compress well even at lower quality.
pub const AVIF_QUALITY_HUGE: f32 = 62.0;

/// Speed preset for tiny images (0-10, higher = faster).
pub const AVIF_SPEED_TINY: u8 = 7;

/// Speed preset for small images.
pub const AVIF_SPEED_SMALL: u8 = 8;

/// Speed preset for medium images.
pub const AVIF_SPEED_MEDIUM: u8 = 7;

/// Speed preset for large images.
pub const AVIF_SPEED_LARGE: u8 = 8;

/// Speed preset for huge images.
pub const AVIF_SPEED_HUGE: u8 = 8;

/// Alpha channel quality for AVIF encoding.
/// Manga pages rarely have transparency. Set to minimum for speed.
pub const AVIF_ALPHA_QUALITY: f32 = 1.0;

/// Pixel count threshold for tiny images (~500x600).
pub const AVIF_TINY_IMAGE_THRESHOLD: u64 = 300_000;

/// Pixel count threshold for small images (~800x1200).
pub const AVIF_SMALL_IMAGE_THRESHOLD: u64 = 1_000_000;

/// Pixel count threshold for medium images (~1400x2100).
/// Most standard manga pages fall in the 1M-3M range.
pub const AVIF_MEDIUM_IMAGE_THRESHOLD: u64 = 3_000_000;

/// Pixel count threshold for large images (~2000x3000).
pub const AVIF_LARGE_IMAGE_THRESHOLD: u64 = 6_000_000;

// ============================================================================
// WebP ENCODING
// ============================================================================

/// Base quality for tiny images (0-100 scale).
pub const WEBP_QUALITY_TINY: f32 = 82.0;

/// Base quality for small images (0-100 scale).
pub const WEBP_QUALITY_SMALL: f32 = 80.0;

/// Base quality for medium images (0-100 scale).
/// Most manga pages - optimized for balance.
pub const WEBP_QUALITY_MEDIUM: f32 = 78.0;

/// Base quality for large images (0-100 scale).
pub const WEBP_QUALITY_LARGE: f32 = 76.0;

/// Base quality for huge images (0-100 scale).
pub const WEBP_QUALITY_HUGE: f32 = 75.0;

/// Compression method for small images (0-6, higher = slower).
/// Note: Currently not used - we use simple encode() for max speed.
pub const WEBP_METHOD_SMALL: u8 = 4;

/// Compression method for medium images (0-6, higher = slower).
pub const WEBP_METHOD_MEDIUM: u8 = 4;

/// Compression method for large images (0-6, higher = slower).
pub const WEBP_METHOD_LARGE: u8 = 4;

/// Pixel count threshold for tiny images (~500x600).
pub const WEBP_TINY_IMAGE_THRESHOLD: u64 = 300_000;

/// Pixel count threshold for small images (~800x1200).
pub const WEBP_SMALL_IMAGE_THRESHOLD: u64 = 1_000_000;

/// Pixel count threshold for medium images (~1400x2100).
pub const WEBP_MEDIUM_IMAGE_THRESHOLD: u64 = 3_000_000;

/// Pixel count threshold for large images (~2000x3000).
pub const WEBP_LARGE_IMAGE_THRESHOLD: u64 = 6_000_000;

// ============================================================================
// PERFORMANCE TUNING
// ============================================================================

/// Minimum chunk size for parallel processing.
/// This is used as a fallback - actual chunk size is calculated dynamically.
pub const MIN_PARALLEL_CHUNK_SIZE: usize = 4;

// ============================================================================
// TUNING PRESETS (for future use)
// ============================================================================
//
// Users could select presets in the future:
//
// FAST PRESET (fastest conversion, larger files):
// - AVIF: Quality 70-75, Speed 9-10
// - WebP: Quality 75-80
//
// BALANCED PRESET (current default - optimal speed+size):
// - AVIF: Quality 70-78, Speed 7-9
// - WebP: Quality 75-82
//
// QUALITY PRESET (best compression, slower):
// - AVIF: Quality 75-82, Speed 6-7
// - WebP: Quality 80-85
//
// ARCHIVE PRESET (maximum compression, very slow):
// - AVIF: Quality 78-85, Speed 4-6
// - WebP: Quality 85-90
