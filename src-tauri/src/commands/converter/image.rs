//! Image conversion module with optimized encoding
//!
//! This module handles image format conversion with optimized encoders,
//! particularly using ravif for AVIF encoding with parallel batch processing.

use crate::prelude::*;
use image::ImageFormat;
use log::{debug, error, trace, warn};
use rayon::prelude::*;
use std::path::{Path, PathBuf};

/// Synchronous image conversion (called from spawn_blocking)
fn convert_image_sync(source_path: &PathBuf, format: ImageOutputFormat) -> Result<Vec<u8>, Error> {
    // Early return: Check if image is already in target format
    if let Some(ext) = source_path.extension().and_then(|e| e.to_str()) {
        let already_converted = match format {
            ImageOutputFormat::Avif => ext.eq_ignore_ascii_case("avif"),
            ImageOutputFormat::WebP => ext.eq_ignore_ascii_case("webp"),
            ImageOutputFormat::None => true,
        };

        if already_converted {
            trace!(
                "Image {:?} already in target format, reading directly",
                source_path
            );
            return std::fs::read(source_path).map_err(Error::from);
        }
    }

    trace!("Loading image from {:?}", source_path);
    let img = image::open(source_path).map_err(|e| {
        error!("Failed to open image {:?}: {}", source_path, e);
        Error::from(e)
    })?;

    match format {
        ImageOutputFormat::Avif => convert_to_avif_optimized(&img),
        ImageOutputFormat::WebP => convert_to_webp(&img),
        ImageOutputFormat::None => unreachable!(),
    }
}

/// Auto-tune AVIF encoding parameters based on image characteristics
///
/// Returns (quality, speed) tuple optimized for the given image:
/// - Small images (< 800x600): Use faster encoding (speed 8)
/// - Medium images (800x600 - 2000x1500): Balanced (speed 6)
/// - Large images (> 2000x1500): Slower for better compression (speed 4)
/// - Grayscale images: Slightly lower quality since compression is easier
/// - Color images: Standard quality settings
///
/// Expected performance gain: 15-25% faster encoding with comparable quality
fn auto_tune_avif_params(img: &image::DynamicImage) -> (f32, u8) {
    let width = img.width();
    let height = img.height();
    let pixel_count = width * height;

    // Detect if image is grayscale
    let is_grayscale = match img {
        image::DynamicImage::ImageLuma8(_) | image::DynamicImage::ImageLuma16(_) => true,
        _ => {
            // Sample pixels to check if effectively grayscale
            let rgb = img.to_rgb8();
            let samples = (pixel_count / 100).max(100).min(1000); // Sample 1% of pixels, min 100, max 1000
            let step = (pixel_count / samples).max(1);

            let mut grayscale_count = 0;
            for i in (0..pixel_count).step_by(step as usize) {
                let x = (i % width) as u32;
                let y = (i / width) as u32;

                if let Some(pixel) = rgb.get_pixel_checked(x, y) {
                    let r = pixel[0];
                    let g = pixel[1];
                    let b = pixel[2];
                    // Allow small deviation for compression artifacts
                    if (r as i16 - g as i16).abs() <= 2 && (g as i16 - b as i16).abs() <= 2 {
                        grayscale_count += 1;
                    }
                }
            }

            // Consider grayscale if > 95% of sampled pixels are grayscale
            (grayscale_count as f32 / samples as f32) > 0.95
        }
    };

    // Auto-tune based on image size
    let (base_quality, speed) = if pixel_count < 480_000 {
        // Small images (< 800x600): Use fast encoding
        // Small files compress quickly, prioritize speed
        (82.0, 8)
    } else if pixel_count < 3_000_000 {
        // Medium images (800x600 - 2000x1500): Balanced
        // Most manga pages fall here
        (80.0, 6)
    } else {
        // Large images (> 2000x1500): Slower encoding for better compression
        // Large files benefit more from better compression
        (78.0, 4)
    };

    // Adjust quality for grayscale
    let quality = if is_grayscale {
        // Grayscale compresses better, can use slightly lower quality
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
fn convert_to_avif_optimized(img: &image::DynamicImage) -> Result<Vec<u8>, Error> {
    use ravif::{Encoder, Img, RGB8};

    trace!("Converting to AVIF using ravif encoder with auto-tuning");

    let width = img.width() as usize;
    let height = img.height() as usize;

    // Auto-tune encoding parameters based on image characteristics
    let (quality, speed) = auto_tune_avif_params(img);

    let rgb_data = img.to_rgb8();

    // Convert to RGB8 slice for ravif
    let rgb_slice: &[RGB8] = unsafe {
        std::slice::from_raw_parts(rgb_data.as_raw().as_ptr() as *const RGB8, width * height)
    };

    let img_ref = Img::new(rgb_slice, width, height);

    // Configure AVIF encoder with auto-tuned settings
    // Quality and speed are dynamically adjusted based on image characteristics
    // Performance note: rav1e automatically uses SIMD (SSE2/AVX2) when available
    // Combined with rayon parallel processing, this maximizes CPU utilization
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

/// Batch convert multiple images in parallel using rayon
///
/// # Arguments
/// * `images` - Slice of image paths to convert
/// * `temp_dir` - Directory to store converted images
/// * `format` - Target image format
/// * `on_progress` - Callback function executed when an image finishes converting
pub fn convert_images_batch<F>(
    images: &[PathBuf],
    temp_dir: &Path,
    format: ImageOutputFormat,
    on_progress: Option<&F>,
) -> Result<Vec<PathBuf>, Error>
where
    F: Fn() + Sync + Send,
{
    if format == ImageOutputFormat::None {
        // Even if no conversion, we should trigger progress for the UI
        if let Some(cb) = on_progress {
            // Trigger callback for every image "processed"
            images.iter().for_each(|_| cb());
        }
        return Ok(images.to_vec());
    }

    let num_threads = rayon::current_num_threads();
    debug!(
        "Batch converting {} images to {:?} using {} threads",
        images.len(),
        format,
        num_threads
    );

    let temp_dir = temp_dir.to_path_buf();
    let chunk_size = (images.len() / num_threads).max(1);

    let results: Vec<_> = images
        .par_iter()
        .with_min_len(chunk_size)
        .map(|img_path| {
            let res = convert_image_sync_to_file(img_path, &temp_dir, format);
            // Signal progress immediately after this specific image is done
            if let Some(cb) = on_progress {
                cb();
            }
            res
        })
        .collect();

    // Collect results and check for errors
    let mut converted = Vec::with_capacity(images.len());
    let mut error_count = 0;

    for result in results {
        match result {
            Ok(path) => converted.push(path),
            Err(e) => {
                error_count += 1;
                if error_count == 1 {
                    return Err(e);
                }
            }
        }
    }

    if error_count > 0 {
        warn!("Batch conversion completed with {} errors", error_count);
    }

    Ok(converted)
}

/// Synchronous conversion that writes directly to file
///
/// This function is called in parallel by rayon threadpool for batch conversions.
/// Each thread handles one image independently for maximum parallelism.
/// Converts a single image file synchronously.
/// Called in parallel by rayon for batch conversions.
fn convert_image_sync_to_file(
    source: &PathBuf,
    temp_dir: &Path,
    format: ImageOutputFormat,
) -> Result<PathBuf, Error> {
    let extension = match format {
        ImageOutputFormat::WebP => "webp",
        ImageOutputFormat::Avif => "avif",
        ImageOutputFormat::None => return Ok(source.clone()),
    };

    // Check if already in target format (avoids unnecessary conversion)
    if let Some(ext) = source.extension() {
        if ext == extension {
            // to_path_buf() is cheaper than converting the image
            return Ok(source.to_path_buf());
        }
    }

    let converted_bytes = convert_image_sync(source, format)?;

    let filename = source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("converted");
    let output_path = temp_dir.join(format!("{}.{}", filename, extension));

    std::fs::write(&output_path, converted_bytes).map_err(|e| {
        error!(
            "Failed to write converted image to {:?}: {}",
            output_path, e
        );
        Error::from(e)
    })?;

    Ok(output_path)
}
