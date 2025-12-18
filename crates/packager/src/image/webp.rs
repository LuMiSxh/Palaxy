//! WebP encoding with auto-tuned parameters.

use super::constants::*;
use common::prelude::*;
use log::trace;

/// Auto-tunes WebP encoding parameters based on image size and content.
#[inline]
pub fn auto_tune_webp_params(img: &image::DynamicImage) -> (f32, u8) {
    let width = img.width();
    let height = img.height();
    let pixel_count = width as u64 * height as u64;

    // Detect if image is grayscale using fast distributed sampling
    let is_grayscale = is_grayscale(img, GrayscaleStrategy::Distributed);

    // Auto-tune based on image size
    let (base_quality, method) = if pixel_count < WEBP_SMALL_IMAGE_THRESHOLD {
        (WEBP_QUALITY_SMALL, WEBP_METHOD_SMALL)
    } else if pixel_count < WEBP_MEDIUM_IMAGE_THRESHOLD {
        (WEBP_QUALITY_MEDIUM, WEBP_METHOD_MEDIUM)
    } else {
        (WEBP_QUALITY_LARGE, WEBP_METHOD_LARGE)
    };

    // Adjust quality for grayscale (it compresses cleaner)
    let quality = if is_grayscale {
        base_quality - WEBP_GRAYSCALE_QUALITY_REDUCTION
    } else {
        base_quality
    };

    trace!(
        "Auto-tuned WebP params for {}x{} {} image: quality={}, method={}",
        width,
        height,
        if is_grayscale { "grayscale" } else { "color" },
        quality,
        method
    );

    (quality, method)
}

/// Converts image to WebP with auto-tuned parameters.
pub fn convert_to_webp(img: &image::DynamicImage) -> Result<Vec<u8>, Error> {
    use webp::{Encoder, WebPMemory};

    trace!("Converting to WebP format with auto-tuning");

    let width = img.width();
    let height = img.height();

    // Auto-tune encoding parameters
    let (quality, _method) = auto_tune_webp_params(img);

    // Convert to RGB or RGBA based on alpha channel presence
    let (encoded, has_alpha): (WebPMemory, bool) = if img.color().has_alpha() {
        let rgba = img.to_rgba8();
        let encoder = Encoder::from_rgba(&rgba, width, height);
        (encoder.encode(quality), true)
    } else {
        // Use Cow to avoid unnecessary conversion if already RGB8
        let rgb = if let Some(rgb_ref) = img.as_rgb8() {
            std::borrow::Cow::Borrowed(rgb_ref)
        } else {
            std::borrow::Cow::Owned(img.to_rgb8())
        };
        let encoder = Encoder::from_rgb(&rgb, width, height);
        (encoder.encode(quality), false)
    };

    trace!(
        "WebP encoding completed, size: {} bytes, alpha: {}",
        encoded.len(),
        has_alpha
    );

    Ok(encoded.to_vec())
}
