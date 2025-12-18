//! AVIF encoding with auto-tuned parameters.

use super::constants::*;
use common::prelude::*;
use log::{error, trace};

/// Auto-tunes AVIF encoding parameters based on image size and content.
#[inline]
pub fn auto_tune_avif_params(img: &image::DynamicImage) -> (f32, u8) {
    let width = img.width();
    let height = img.height();
    let pixel_count = width as u64 * height as u64;

    // Detect if image is grayscale using fast distributed sampling
    let is_grayscale = is_grayscale(img, GrayscaleStrategy::Distributed);

    // Auto-tune based on image size
    let (base_quality, speed) = if pixel_count < AVIF_SMALL_IMAGE_THRESHOLD {
        (AVIF_QUALITY_SMALL, AVIF_SPEED_SMALL)
    } else if pixel_count < AVIF_MEDIUM_IMAGE_THRESHOLD {
        (AVIF_QUALITY_MEDIUM, AVIF_SPEED_MEDIUM)
    } else {
        (AVIF_QUALITY_LARGE, AVIF_SPEED_LARGE)
    };

    // Adjust quality for grayscale (it compresses cleaner)
    let quality = if is_grayscale {
        base_quality - AVIF_GRAYSCALE_QUALITY_REDUCTION
    } else {
        base_quality
    };

    trace!(
        "Auto-tuned AVIF params for {}x{} {} image: quality={}, speed={}",
        width,
        height,
        if is_grayscale { "grayscale" } else { "color" },
        quality,
        speed
    );

    (quality, speed)
}

/// Converts image to AVIF with auto-tuned parameters.
pub fn convert_to_avif(img: &image::DynamicImage) -> Result<Vec<u8>, Error> {
    use ravif::{Encoder, Img, RGB8};

    trace!("Converting to AVIF using ravif encoder with auto-tuning");

    let width = img.width() as usize;
    let height = img.height() as usize;

    // Auto-tune encoding parameters
    let (quality, speed) = auto_tune_avif_params(img);

    // Convert to RGB8 if necessary - use Cow to avoid unnecessary clones
    let rgb_cow = if let Some(rgb_ref) = img.as_rgb8() {
        std::borrow::Cow::Borrowed(rgb_ref)
    } else {
        std::borrow::Cow::Owned(img.to_rgb8())
    };

    // Reinterpret as RGB8 slice for ravif
    // SAFETY: RGB8 and [u8; 3] have the same memory layout
    let rgb_slice: &[RGB8] = unsafe {
        std::slice::from_raw_parts(rgb_cow.as_raw().as_ptr() as *const RGB8, width * height)
    };

    let img_ref = Img::new(rgb_slice, width, height);

    // Configure AVIF encoder with optimized settings
    let encoder = Encoder::new()
        .with_quality(quality)
        .with_speed(speed)
        .with_alpha_quality(quality);

    let encoded = encoder.encode_rgb(img_ref).map_err(|e| {
        error!("Failed to encode image with ravif: {}", e);
        Error::Unsupported(format!("AVIF encoding failed: {}", e))
    })?;

    trace!(
        "AVIF encoding completed, size: {} bytes",
        encoded.avif_file.len()
    );
    Ok(encoded.avif_file)
}
