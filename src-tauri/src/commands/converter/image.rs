//! Image conversion module with optimized encoding
//!
//! This module handles image format conversion with optimized encoders,
//! particularly using ravif for faster AVIF encoding.

use crate::prelude::*;
use image::ImageFormat;
use log::{debug, error, trace};
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::task::spawn_blocking;

/// Converts an image to the specified output format using optimized encoders.
///
/// # Arguments
/// * `image_path` - Path to the source image
/// * `temp_dir` - Directory to store the converted image
/// * `format` - Target image format (WebP or AVIF)
///
/// # Returns
/// * `Result<PathBuf, Error>` - Path to the converted image
pub async fn convert_image(
    image_path: &PathBuf,
    temp_dir: &Path,
    format: ImageOutputFormat,
) -> Result<PathBuf, Error> {
    // Determine target format and extension
    let extension = match format {
        ImageOutputFormat::WebP => "webp",
        ImageOutputFormat::Avif => "avif",
        ImageOutputFormat::None => return Ok(image_path.clone()),
    };

    // Check if already in target format
    if let Some(ext) = image_path.extension() {
        if ext == extension {
            trace!(
                "Image {:?} is already in {:?} format, skipping conversion",
                image_path, format
            );
            return Ok(image_path.clone());
        }
    }

    debug!("Converting image {:?} to {:?} format", image_path, format);

    // Clone the path for use in blocking task
    let source_path = image_path.clone();
    let temp_dir = temp_dir.to_path_buf();

    // Perform image conversion in a blocking task
    let converted_bytes = spawn_blocking(move || convert_image_sync(&source_path, format))
        .await
        .map_err(|e| {
            error!("Async task failed during image conversion: {}", e);
            Error::AsyncTaskError(e.to_string())
        })??;

    // Create output path
    let filename = image_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("converted");
    let output_path = temp_dir.join(format!("{}.{}", filename, extension));

    trace!("Writing {:?} image to {:?}", format, output_path);
    let mut file = File::create(&output_path).await.map_err(|e| {
        error!("Failed to create output file {:?}: {}", output_path, e);
        Error::from(e)
    })?;

    file.write_all(&converted_bytes).await.map_err(|e| {
        error!("Failed to write converted data to file: {}", e);
        Error::from(e)
    })?;

    file.flush().await.map_err(|e| {
        error!("Failed to flush file: {}", e);
        Error::from(e)
    })?;

    debug!(
        "Successfully converted image to {:?}: {:?}",
        format, output_path
    );
    Ok(output_path)
}

/// Synchronous image conversion (called from spawn_blocking)
fn convert_image_sync(source_path: &PathBuf, format: ImageOutputFormat) -> Result<Vec<u8>, Error> {
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

/// Convert image to AVIF using optimized ravif encoder
fn convert_to_avif_optimized(img: &image::DynamicImage) -> Result<Vec<u8>, Error> {
    use ravif::{Encoder, Img, RGB8};

    trace!("Converting to AVIF using ravif encoder");

    let width = img.width() as usize;
    let height = img.height() as usize;

    let rgb_data = img.to_rgb8();

    // Convert to RGB8 slice for ravif
    let rgb_slice: &[RGB8] = unsafe {
        std::slice::from_raw_parts(rgb_data.as_raw().as_ptr() as *const RGB8, width * height)
    };

    let img_ref = Img::new(rgb_slice, width, height);

    let encoder = Encoder::new()
        .with_quality(80.0) // Good balance between quality and file size
        .with_speed(6); // Balance between speed and compression (1-10, higher is faster)

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
/// * `images` - Vector of image paths to convert
/// * `temp_dir` - Directory to store converted images
/// * `format` - Target image format
///
/// # Returns
/// * `Result<Vec<PathBuf>, Error>` - Paths to converted images
#[allow(dead_code)]
pub fn convert_images_batch_sync(
    images: Vec<PathBuf>,
    temp_dir: &Path,
    format: ImageOutputFormat,
) -> Result<Vec<PathBuf>, Error> {
    use rayon::prelude::*;

    if format == ImageOutputFormat::None {
        return Ok(images);
    }

    debug!("Batch converting {} images to {:?}", images.len(), format);

    let temp_dir = temp_dir.to_path_buf();
    let results: Vec<_> = images
        .par_iter()
        .map(|img_path| convert_image_sync_to_file(img_path, &temp_dir, format))
        .collect();

    results.into_iter().collect()
}

/// Synchronous conversion that writes directly to file
#[allow(dead_code)]
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

    // Check if already in target format
    if let Some(ext) = source.extension() {
        if ext == extension {
            return Ok(source.clone());
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
