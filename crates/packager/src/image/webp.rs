//! WebP encoding with enhanced auto-tuning algorithm.
//!
//! This module implements a 5-tier adaptive encoding system optimized for
//! both conversion speed AND file size, matching the AVIF encoder approach.

use super::constants::*;
use common::prelude::*;
use log::trace;

/// Auto-tunes WebP encoding parameters based on image size
#[inline]
pub fn auto_tune_webp_params(img: &image::DynamicImage, is_grayscale: bool) -> (f32, u8) {
    let width = img.width();
    let height = img.height();
    let pixel_count = width as u64 * height as u64;

    let (mut quality, method) = if pixel_count < WEBP_TINY_IMAGE_THRESHOLD {
        (WEBP_QUALITY_TINY, WEBP_METHOD_SMALL)
    } else if pixel_count < WEBP_SMALL_IMAGE_THRESHOLD {
        (WEBP_QUALITY_SMALL, WEBP_METHOD_SMALL)
    } else if pixel_count < WEBP_MEDIUM_IMAGE_THRESHOLD {
        (WEBP_QUALITY_MEDIUM, WEBP_METHOD_MEDIUM)
    } else if pixel_count < WEBP_LARGE_IMAGE_THRESHOLD {
        (WEBP_QUALITY_LARGE, WEBP_METHOD_LARGE)
    } else {
        (WEBP_QUALITY_HUGE, WEBP_METHOD_LARGE)
    };

    // Optimization for grayscale
    if is_grayscale {
        // Grayscale compresses very well, we can slightly lower quality to save space
        // without visible artifacts
        quality = (quality - 2.0).max(60.0);
    }

    trace!(
        "WebP auto-tune: {}x{} ({:.2}MP, gray={}) → quality={}, method={} [{}]",
        width,
        height,
        pixel_count as f64 / 1_000_000.0,
        is_grayscale,
        quality,
        method,
        size_tier_name(pixel_count)
    );

    (quality, method)
}

/// Returns human-readable name for the size tier.
#[inline]
fn size_tier_name(pixel_count: u64) -> &'static str {
    if pixel_count < WEBP_TINY_IMAGE_THRESHOLD {
        "tiny"
    } else if pixel_count < WEBP_SMALL_IMAGE_THRESHOLD {
        "small"
    } else if pixel_count < WEBP_MEDIUM_IMAGE_THRESHOLD {
        "medium"
    } else if pixel_count < WEBP_LARGE_IMAGE_THRESHOLD {
        "large"
    } else {
        "huge"
    }
}

/// Converts image to WebP with auto-tuned parameters.
pub fn convert_to_webp(img: &image::DynamicImage) -> Result<Vec<u8>, Error> {
    use webp::{Encoder, WebPMemory};

    let width = img.width();
    let height = img.height();

    // 1. Detect Grayscale
    let is_grayscale =
        common::image_utils::is_grayscale(img, common::image_utils::GrayscaleStrategy::Distributed);

    // 2. Auto-tune
    let (quality, _method) = auto_tune_webp_params(img, is_grayscale);

    trace!(
        "Converting {}x{} image to WebP (quality={}, gray={})",
        width,
        height,
        quality,
        is_grayscale
    );

    let (encoded, _has_alpha): (WebPMemory, bool) = if img.color().has_alpha() {
        let rgba = img.to_rgba8();
        let encoder = Encoder::from_rgba(&rgba, width, height);
        (encoder.encode(quality), true)
    } else {
        let rgb = if let Some(rgb_ref) = img.as_rgb8() {
            std::borrow::Cow::Borrowed(rgb_ref)
        } else {
            std::borrow::Cow::Owned(img.to_rgb8())
        };
        let encoder = Encoder::from_rgb(&rgb, width, height);
        (encoder.encode(quality), false)
    };

    Ok(encoded.to_vec())
}
