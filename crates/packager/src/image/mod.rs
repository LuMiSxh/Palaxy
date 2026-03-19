//! Image processing with optimized encoding.

mod avif;
mod constants;
mod webp;

pub use avif::convert_to_avif;
pub use constants::*;
pub use webp::convert_to_webp;

use common::prelude::*;
use log::trace;
use memmap2::Mmap;
use rayon::prelude::*;
use std::fs::File;
use std::path::PathBuf;

/// Processed image page ready for packaging.
pub struct ProcessedPage {
    pub data: Vec<u8>,
    pub extension: String,
}

impl ProcessedPage {
    /// Creates a new processed page with pre-allocated extension string.
    #[inline]
    fn new(data: Vec<u8>, extension: &str) -> Self {
        Self {
            data,
            extension: extension.to_string(),
        }
    }
}

/// Batch converts images to specified format with optional progress callback.
///
/// # Arguments
/// * `images` - Paths to images to process
/// * `format` - Target image format
/// * `on_progress` - Optional callback for progress tracking
/// * `rayon_chunk_size` - Optional chunk size for Rayon parallelism. If None, calculates optimal size.
pub fn process_images_to_memory<F>(
    images: &[PathBuf],
    format: ImageOutputFormat,
    on_progress: Option<&F>,
    rayon_chunk_size: Option<usize>,
) -> Result<Vec<ProcessedPage>, Error>
where
    F: Fn() + Sync + Send,
{
    process_images_to_memory_with_options(images, format, on_progress, rayon_chunk_size, None)
}

/// Batch converts images with full control over options including downscaling.
///
/// # Arguments
/// * `max_dimension` - Optional max width/height. Images exceeding this are downscaled preserving aspect ratio.
pub fn process_images_to_memory_with_options<F>(
    images: &[PathBuf],
    format: ImageOutputFormat,
    on_progress: Option<&F>,
    rayon_chunk_size: Option<usize>,
    max_dimension: Option<u32>,
) -> Result<Vec<ProcessedPage>, Error>
where
    F: Fn() + Sync + Send,
{
    let num_threads = rayon::current_num_threads();

    // Calculate optimal chunk size for work distribution
    // Target: 3x chunks per thread for good load balancing
    let chunk_size = rayon_chunk_size.unwrap_or_else(|| {
        let target_chunks = num_threads * 3;
        (images.len() / target_chunks)
            .max(MIN_PARALLEL_CHUNK_SIZE)
            .min(32)
    });

    // Pre-allocate result vector to avoid reallocations
    let results: Vec<_> = images
        .par_iter()
        .with_min_len(chunk_size)
        .map(|img_path| {
            // Process single image
            let result = process_single_image(img_path, format, max_dimension);

            // Trigger progress callback
            if let Some(cb) = on_progress {
                cb();
            }
            result
        })
        .collect();

    // Collect results, propagating errors early
    results.into_iter().collect()
}

/// Processes single image with passthrough or format conversion.
///
/// # Arguments
/// * `max_dimension` - Optional max width/height. Images exceeding this are downscaled.
pub(crate) fn process_single_image(
    path: &PathBuf,
    format: ImageOutputFormat,
    max_dimension: Option<u32>,
) -> Result<ProcessedPage, Error> {
    // Check if source format matches target format first (before file I/O)
    let source_ext = path.extension().and_then(|e| e.to_str()).unwrap_or("jpg");

    // Handle "None" (Original/Passthrough) case - fastest path
    if format == ImageOutputFormat::None {
        let data = std::fs::read(path).map_err(Error::from)?;
        return Ok(ProcessedPage::new(data, source_ext));
    }

    // Determine target extension
    let target_ext = match format {
        ImageOutputFormat::Avif => "avif",
        ImageOutputFormat::WebP => "webp",
        ImageOutputFormat::None => unreachable!(),
    };

    // Skip conversion if source already matches target and no downscaling needed
    if source_ext.eq_ignore_ascii_case(target_ext) && max_dimension.is_none() {
        trace!(
            "Source matches target format ({}), skipping conversion",
            source_ext
        );
        let data = std::fs::read(path).map_err(Error::from)?;
        return Ok(ProcessedPage::new(data, target_ext));
    }

    // Load and convert image
    let file = File::open(path).map_err(Error::from)?;
    let mmap = unsafe { Mmap::map(&file).map_err(Error::from)? };
    let mut img = image::load_from_memory(&mmap).map_err(Error::from)?;

    // Optional downscaling
    if let Some(max_dim) = max_dimension {
        let w = img.width();
        let h = img.height();
        if w > max_dim || h > max_dim {
            let scale = max_dim as f64 / w.max(h) as f64;
            let new_w = (w as f64 * scale) as u32;
            let new_h = (h as f64 * scale) as u32;
            trace!(
                "Downscaling {}x{} -> {}x{} (max_dim={})",
                w, h, new_w, new_h, max_dim
            );
            img = img.resize(new_w, new_h, image::imageops::FilterType::Lanczos3);
        }
    }

    // Perform conversion with auto-tuned parameters
    let data = match format {
        ImageOutputFormat::Avif => avif::convert_to_avif(&img)?,
        ImageOutputFormat::WebP => webp::convert_to_webp(&img)?,
        ImageOutputFormat::None => unreachable!(),
    };

    Ok(ProcessedPage::new(data, target_ext))
}
