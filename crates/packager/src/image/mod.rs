//! Image processing with optimized encoding.

mod avif;
mod constants;
mod webp;

pub use constants::*;

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
pub fn process_images_to_memory<F>(
    images: &[PathBuf],
    format: ImageOutputFormat,
    on_progress: Option<&F>,
) -> Result<Vec<ProcessedPage>, Error>
where
    F: Fn() + Sync + Send,
{
    let num_threads = rayon::current_num_threads();
    let chunk_size = (images.len() / num_threads).max(MIN_PARALLEL_CHUNK_SIZE);

    // Pre-allocate result vector to avoid reallocations
    let results: Vec<_> = images
        .par_iter()
        .with_min_len(chunk_size)
        .map(|img_path| {
            // Process single image
            let result = process_single_image(img_path, format);

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
fn process_single_image(path: &PathBuf, format: ImageOutputFormat) -> Result<ProcessedPage, Error> {
    // Check if source format matches target format first (before file I/O)
    let source_ext = path.extension().and_then(|e| e.to_str()).unwrap_or("jpg");

    // Handle "None" (Original/Passthrough) case - fastest path
    if format == ImageOutputFormat::None {
        let file = File::open(path).map_err(Error::from)?;
        let mmap = unsafe { Mmap::map(&file).map_err(Error::from)? };
        return Ok(ProcessedPage::new(mmap.to_vec(), source_ext));
    }

    // Determine target extension
    let target_ext = match format {
        ImageOutputFormat::Avif => "avif",
        ImageOutputFormat::WebP => "webp",
        ImageOutputFormat::None => unreachable!(),
    };

    // Skip conversion if source already matches target
    if source_ext.eq_ignore_ascii_case(target_ext) {
        trace!(
            "Source matches target format ({}), skipping conversion",
            source_ext
        );
        let file = File::open(path).map_err(Error::from)?;
        let mmap = unsafe { Mmap::map(&file).map_err(Error::from)? };
        return Ok(ProcessedPage::new(mmap.to_vec(), target_ext));
    }

    // Load and convert image
    let file = File::open(path).map_err(Error::from)?;
    let mmap = unsafe { Mmap::map(&file).map_err(Error::from)? };
    let img = image::load_from_memory(&mmap).map_err(Error::from)?;

    // Perform conversion with auto-tuned parameters
    let data = match format {
        ImageOutputFormat::Avif => avif::convert_to_avif(&img)?,
        ImageOutputFormat::WebP => webp::convert_to_webp(&img)?,
        ImageOutputFormat::None => unreachable!(),
    };

    Ok(ProcessedPage::new(data, target_ext))
}
