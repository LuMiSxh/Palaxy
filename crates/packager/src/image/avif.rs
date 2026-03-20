//! AVIF encoding with enhanced auto-tuning algorithm.
//!
//! This module implements a 5-tier adaptive encoding system optimized for
//! both conversion speed AND file size. The algorithm analyzes image dimensions
//! to select optimal quality and speed parameters based on encoder characteristics.
//!
//! For grayscale images (typical manga pages), a specialized encoding path feeds
//! the luma channel as Y with neutral chroma (Cb=Cr=128), allowing the encoder
//! to compress chroma planes to near-zero overhead.

use super::constants::*;
use common::prelude::*;
use log::{error, trace};

/// Auto-tunes AVIF encoding parameters based on image size and content.
#[inline]
pub fn auto_tune_avif_params(img: &image::DynamicImage, is_grayscale: bool) -> (f32, u8) {
    let width = img.width();
    let height = img.height();
    let pixel_count = width as u64 * height as u64;

    // 5-tier adaptive tuning based on image size
    let (mut quality, mut speed) = if pixel_count < AVIF_TINY_IMAGE_THRESHOLD {
        (AVIF_QUALITY_TINY, AVIF_SPEED_TINY)
    } else if pixel_count < AVIF_SMALL_IMAGE_THRESHOLD {
        (AVIF_QUALITY_SMALL, AVIF_SPEED_SMALL)
    } else if pixel_count < AVIF_MEDIUM_IMAGE_THRESHOLD {
        (AVIF_QUALITY_MEDIUM, AVIF_SPEED_MEDIUM)
    } else if pixel_count < AVIF_LARGE_IMAGE_THRESHOLD {
        (AVIF_QUALITY_LARGE, AVIF_SPEED_LARGE)
    } else {
        (AVIF_QUALITY_HUGE, AVIF_SPEED_HUGE)
    };

    // Optimization: Grayscale manga compresses exceptionally well.
    // We can lower quality significantly and reduce speed for better compression
    // without visible artifacts on line art / grayscale content.
    if is_grayscale {
        // Lower speed = slower encoding but better compression ratio.
        // Worth it for grayscale since the content is simpler.
        speed = speed.saturating_sub(1).max(6);
        // Grayscale line art is very resilient to quality reduction.
        // -5 quality saves significant space with no visible loss on manga.
        quality = (quality - AVIF_GRAYSCALE_QUALITY_REDUCTION).max(55.0);
    }

    trace!(
        "AVIF auto-tune: {}x{} ({:.2}MP, gray={}) → quality={}, speed={} [{}]",
        width,
        height,
        pixel_count as f64 / 1_000_000.0,
        is_grayscale,
        quality,
        speed,
        size_tier_name(pixel_count)
    );

    (quality, speed)
}

fn size_tier_name(pixel_count: u64) -> &'static str {
    if pixel_count < AVIF_TINY_IMAGE_THRESHOLD {
        "tiny"
    } else if pixel_count < AVIF_SMALL_IMAGE_THRESHOLD {
        "small"
    } else if pixel_count < AVIF_MEDIUM_IMAGE_THRESHOLD {
        "medium"
    } else if pixel_count < AVIF_LARGE_IMAGE_THRESHOLD {
        "large"
    } else {
        "huge"
    }
}

pub fn convert_to_avif(img: &image::DynamicImage) -> Result<Vec<u8>, Error> {
    let width = img.width() as usize;
    let height = img.height() as usize;

    // 1. Detect Grayscale (Fast Strategy)
    // This overhead (~10-20ms) saves seconds in encoding time.
    let is_grayscale =
        common::image_utils::is_grayscale(img, common::image_utils::GrayscaleStrategy::Distributed);

    // 2. Auto-tune parameters
    let (quality, speed) = auto_tune_avif_params(img, is_grayscale);

    trace!(
        "Converting {}x{} image to AVIF (quality={}, speed={}, gray={})",
        width,
        height,
        quality,
        speed,
        is_grayscale
    );

    // Single-threaded per encode: rayon handles parallelism across images.
    // Without this, rav1e spawns N threads per encode × M rayon workers = thread explosion.
    let encoder = ravif::Encoder::new()
        .with_quality(quality)
        .with_speed(speed)
        .with_alpha_quality(AVIF_ALPHA_QUALITY)
        .with_num_threads(Some(1));

    if is_grayscale {
        // Grayscale optimization: Feed luma as Y with neutral chroma (Cb=Cr=128).
        // The encoder compresses the constant chroma planes to near-zero,
        // producing significantly smaller files than encoding as RGB.
        encode_grayscale_avif(&encoder, img, width, height)
    } else {
        encode_color_avif(&encoder, img, width, height)
    }
}

/// Encodes a grayscale image using raw YCbCr planes with neutral chroma.
/// The constant Cb/Cr=128 planes compress to near-zero overhead.
fn encode_grayscale_avif(
    encoder: &ravif::Encoder,
    img: &image::DynamicImage,
    width: usize,
    height: usize,
) -> Result<Vec<u8>, Error> {
    // Extract luma channel directly — avoids full RGB conversion
    let luma = img.to_luma8();
    let luma_bytes = luma.as_raw();

    // Feed as YCbCr: Y=luma, Cb=128, Cr=128 (neutral chroma)
    let planes = luma_bytes.iter().map(|&y| [y, 128u8, 128u8]);

    let encoded = encoder
        .encode_raw_planes_8_bit(
            width,
            height,
            planes,
            None::<[u8; 0]>,
            rav1e::prelude::PixelRange::Full,
            ravif::MatrixCoefficients::BT601,
        )
        .map_err(|e| {
            error!("AVIF grayscale encoding failed: {}", e);
            Error::Unsupported(format!("AVIF encoding failed: {}", e))
        })?;

    Ok(encoded.avif_file)
}

/// Encodes a color image using the standard RGB path.
fn encode_color_avif(
    encoder: &ravif::Encoder,
    img: &image::DynamicImage,
    width: usize,
    height: usize,
) -> Result<Vec<u8>, Error> {
    use ravif::{Img, RGB8};

    let rgb_cow = if let Some(rgb_ref) = img.as_rgb8() {
        std::borrow::Cow::Borrowed(rgb_ref)
    } else {
        std::borrow::Cow::Owned(img.to_rgb8())
    };

    let rgb_slice: &[RGB8] = unsafe {
        std::slice::from_raw_parts(rgb_cow.as_raw().as_ptr() as *const RGB8, width * height)
    };

    let img_ref = Img::new(rgb_slice, width, height);

    let encoded = encoder.encode_rgb(img_ref).map_err(|e| {
        error!("AVIF encoding failed: {}", e);
        Error::Unsupported(format!("AVIF encoding failed: {}", e))
    })?;

    Ok(encoded.avif_file)
}
