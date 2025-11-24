//! Image conversion module with optimized encoding
//!
//! This module handles image format conversion with optimized encoders,
//! particularly using ravif for AVIF encoding with parallel batch processing.

use crate::prelude::*;
use image::{GenericImageView, ImageFormat, Pixel};
use log::{error, trace};
use memmap2::Mmap;
use rayon::prelude::*;
use std::fs::File;
use std::path::PathBuf;

pub struct ProcessedPage {
    pub data: Vec<u8>,
    pub extension: String,
}

/// Batch convert images in memory
pub fn process_images_to_memory<F>(
    images: &[PathBuf],
    format: ImageOutputFormat,
    on_progress: Option<&F>,
) -> Result<Vec<ProcessedPage>, Error>
where
    F: Fn() + Sync + Send,
{
    let num_threads = rayon::current_num_threads();
    let chunk_size = (images.len() / num_threads).max(1);

    let results: Vec<_> = images
        .par_iter()
        .with_min_len(chunk_size)
        .map(|img_path| {
            // 1. Determine if we convert or passthrough
            let result = process_single_image(img_path, format);

            // 2. Trigger Progress
            if let Some(cb) = on_progress {
                cb();
            }
            result
        })
        .collect();

    // Unwrap results
    let mut pages = Vec::with_capacity(images.len());
    for res in results {
        pages.push(res?);
    }
    Ok(pages)
}

fn process_single_image(path: &PathBuf, format: ImageOutputFormat) -> Result<ProcessedPage, Error> {
    let file = File::open(path).map_err(Error::from)?;

    // Memory map the input file
    // This avoids a heap allocation and copy for the initial read
    let mmap = unsafe { Mmap::map(&file).map_err(Error::from)? };

    // Handle "None" (Original) Case
    if format == ImageOutputFormat::None {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("jpg")
            .to_string();

        return Ok(ProcessedPage {
            data: mmap.to_vec(), // Efficient copy from memory map
            extension,
        });
    }

    // Check if target is same as source (e.g. source is webp, target is webp)
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let target_ext = match format {
            ImageOutputFormat::Avif => "avif",
            ImageOutputFormat::WebP => "webp",
            _ => "",
        };

        if ext.eq_ignore_ascii_case(target_ext) {
            trace!(
                "Source matches target format ({}), skipping conversion",
                ext
            );
            return Ok(ProcessedPage {
                data: mmap.to_vec(),
                extension: target_ext.to_string(),
            });
        }
    }

    // Handle Conversion Case
    // image::load_from_memory is efficient with slices (uses the mmap directly)
    let img = image::load_from_memory(&mmap).map_err(Error::from)?;

    // Perform Conversion
    let (data, extension) = match format {
        ImageOutputFormat::Avif => (convert_to_avif(&img)?, "avif".to_string()),
        ImageOutputFormat::WebP => (convert_to_webp(&img)?, "webp".to_string()),
        ImageOutputFormat::None => unreachable!(),
    };

    Ok(ProcessedPage { data, extension })
}

/// Auto-tune AVIF encoding parameters based on image characteristics
///
/// Returns (quality, speed) tuple optimized for the given image:
/// - Small images (< 480k pixels): Speed 8 (Fastest)
/// - Medium images (< 3M pixels): Speed 7 (Balanced)
/// - Large images (> 3M pixels): Speed 6 (Better compression, but avoids Speed 4 cliff)
fn auto_tune_avif_params(img: &image::DynamicImage) -> (f32, u8) {
    let width = img.width();
    let height = img.height();
    let pixel_count = width as u64 * height as u64;

    // Zero-allocation grayscale detection
    // We sample a fixed number of pixels instead of converting the whole image
    let is_grayscale = match img {
        image::DynamicImage::ImageLuma8(_) | image::DynamicImage::ImageLuma16(_) => true,
        _ => {
            // Sample ~500 pixels distributed across the image
            let samples = 500;
            let step = (pixel_count / samples).max(1);

            let mut grayscale_samples = 0;
            let mut total_sampled = 0;

            // Use GenericImageView to avoid allocation
            // We simulate a flat iteration
            for i in (0..pixel_count).step_by(step as usize) {
                let x = (i % width as u64) as u32;
                let y = (i / width as u64) as u32;

                let pixel = img.get_pixel(x, y);
                let channels = pixel.channels();

                if channels.len() >= 3 {
                    let r = channels[0];
                    let g = channels[1];
                    let b = channels[2];

                    // Allow small compression artifact deviation
                    // Using u8 comparison (casts happen implicitly for diff)
                    if r.abs_diff(g) <= 3 && g.abs_diff(b) <= 3 {
                        grayscale_samples += 1;
                    }
                    total_sampled += 1;
                }
            }

            // If > 95% of sampled pixels are grayscale, treat as grayscale
            total_sampled > 0 && (grayscale_samples as f32 / total_sampled as f32) > 0.95
        }
    };

    // Auto-tune based on image size
    // Note: Speed 4 is exponentially slower than Speed 6.
    // We cap minimum speed at 6 to maintain responsiveness.
    let (base_quality, speed) = if pixel_count < 480_000 {
        // Small images (< 800x600)
        (82.0, 8)
    } else if pixel_count < 3_000_000 {
        // Medium images (Standard Manga Page)
        (80.0, 7)
    } else {
        // Large images (Double spreads / High Res)
        // Speed 6 is the sweet spot for rav1e. Speed 4 is too slow for batching.
        (78.0, 6)
    };

    // Adjust quality for grayscale (it compresses cleaner)
    let quality = if is_grayscale {
        base_quality - 2.0
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

/// Convert image to AVIF using optimized ravif encoder with auto-tuning
fn convert_to_avif(img: &image::DynamicImage) -> Result<Vec<u8>, Error> {
    use ravif::{Encoder, Img, RGB8};

    trace!("Converting to AVIF using ravif encoder with auto-tuning");

    let width = img.width() as usize;
    let height = img.height() as usize;

    // Auto-tune encoding parameters based on the original image
    let (quality, speed) = auto_tune_avif_params(img);

    let rgb_cow = if let Some(rgb_ref) = img.as_rgb8() {
        std::borrow::Cow::Borrowed(rgb_ref)
    } else {
        // Only allocate if conversion (e.g. from RGBA -> RGB) is strictly necessary
        std::borrow::Cow::Owned(img.to_rgb8())
    };

    let rgb_slice: &[RGB8] = unsafe {
        std::slice::from_raw_parts(rgb_cow.as_raw().as_ptr() as *const RGB8, width * height)
    };

    let img_ref = Img::new(rgb_slice, width, height);

    // Configure AVIF encoder
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

/// Convert image to WebP using standard image crate
fn convert_to_webp(img: &image::DynamicImage) -> Result<Vec<u8>, Error> {
    trace!("Converting to WebP format");

    let mut converted_data = std::io::Cursor::new(Vec::new());
    img.write_to(&mut converted_data, ImageFormat::WebP)
        .map_err(|e| {
            error!("Failed to encode image to WebP: {}", e);
            Error::from(e)
        })?;

    let bytes = converted_data.into_inner();
    trace!("WebP encoding completed, size: {} bytes", bytes.len());
    Ok(bytes)
}
