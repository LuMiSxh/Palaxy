//! Streaming encode-and-write pipeline.
//!
//! Encodes images in parallel via rayon and writes them to a generator
//! as they complete, using a bounded channel to limit peak memory.

use crate::generator::Generator;
use crate::image::{
    process_images_to_memory_with_options, process_single_image, ProcessedPage,
};
use common::prelude::*;
use crossbeam_channel::bounded;
use log::debug;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Processes images in parallel and writes them to a generator in order.
///
/// Uses a bounded channel to overlap encoding with I/O while limiting
/// peak memory to `buffer_size` encoded images in flight.
///
/// For small batches (<= buffer_size), falls back to the in-memory approach.
pub fn process_and_write_streaming<G>(
    images: &[PathBuf],
    format: ImageOutputFormat,
    generator: &mut G,
    on_progress: Option<Arc<dyn Fn() + Send + Sync>>,
    buffer_size: usize,
) -> Result<(), Error>
where
    G: Generator,
{
    process_and_write_streaming_with_options(
        images,
        format,
        generator,
        on_progress,
        buffer_size,
        None,
    )
}

/// Processes images in parallel and writes them to a generator in order,
/// with optional downscaling.
pub fn process_and_write_streaming_with_options<G>(
    images: &[PathBuf],
    format: ImageOutputFormat,
    generator: &mut G,
    on_progress: Option<Arc<dyn Fn() + Send + Sync>>,
    buffer_size: usize,
    max_dimension: Option<u32>,
) -> Result<(), Error>
where
    G: Generator,
{
    // For small batches, use the simpler in-memory approach
    if images.len() <= buffer_size {
        debug!(
            "Small batch ({} images), using in-memory approach",
            images.len()
        );
        let progress_ref = on_progress.as_ref().map(|arc| {
            let arc = Arc::clone(arc);
            move || arc()
        });
        let pages = process_images_to_memory_with_options(
            images,
            format,
            progress_ref.as_ref(),
            None,
            max_dimension,
        )?;
        for page in pages {
            generator.add_page_from_memory(&page.data, &page.extension)?;
        }
        return Ok(());
    }

    debug!(
        "Streaming pipeline: {} images, buffer_size={}, max_dim={:?}",
        images.len(),
        buffer_size,
        max_dimension
    );

    // Bounded channel limits peak memory
    let (tx, rx) = bounded::<(usize, Result<ProcessedPage, Error>)>(buffer_size);

    // Clone images for the encoding thread
    let images_owned: Vec<PathBuf> = images.to_vec();
    let progress = on_progress.clone();

    // Spawn encoding work on a separate thread so we can consume concurrently
    let handle = std::thread::spawn(move || {
        images_owned
            .par_iter()
            .enumerate()
            .for_each(|(idx, path)| {
                let result = process_single_image(path, format, max_dimension);
                // If the receiver is dropped (e.g., on error), stop sending
                let _ = tx.send((idx, result));
                if let Some(ref cb) = progress {
                    cb();
                }
            });
        // tx is dropped here, closing the channel
    });

    // Consume encoded pages and write in order
    let mut next_to_write: usize = 0;
    let mut pending: BTreeMap<usize, ProcessedPage> = BTreeMap::new();

    for (idx, result) in rx {
        let page = result?;

        if idx == next_to_write {
            // Write immediately
            generator.add_page_from_memory(&page.data, &page.extension)?;
            next_to_write += 1;

            // Drain consecutive buffered pages
            while let Some(buffered) = pending.remove(&next_to_write) {
                generator.add_page_from_memory(&buffered.data, &buffered.extension)?;
                next_to_write += 1;
            }
        } else {
            // Buffer out-of-order page
            pending.insert(idx, page);
        }
    }

    // Wait for the encoding thread to finish
    handle
        .join()
        .map_err(|_| Error::Unsupported("Encoding thread panicked".into()))?;

    debug!(
        "Streaming pipeline complete: wrote {} pages",
        next_to_write
    );

    Ok(())
}
