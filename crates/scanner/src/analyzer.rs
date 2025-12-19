//! Volume detection and image analysis.

use std::path::PathBuf;
use std::sync::Arc;

use log::{debug, error, info, trace};
use rayon::prelude::*;
use tauri::async_runtime::{spawn, spawn_blocking, JoinHandle};
use tokio::sync::Semaphore;

use common::prelude::*;

/// Identifies volume start chapters by analyzing cover images.
pub async fn determine_volume_start_chapters(
    images_per_chapter: Vec<Vec<PathBuf>>,
    sensibility: f64,
) -> EResult<Vec<usize>> {
    info!(
        "Determining volume start chapters with sensitivity: {:.2}",
        sensibility
    );

    let semaphore = Arc::new(Semaphore::new(num_cpus::get()));
    debug!("Using {} threads for grayscale analysis", num_cpus::get());

    // Pre-allocate with estimated capacity
    let estimated_volumes = (images_per_chapter.len() / 10).max(1);
    let book_start_chapters = Arc::new(tokio::sync::Mutex::new(Vec::with_capacity(
        estimated_volumes,
    )));

    let handles: Vec<JoinHandle<EResult<()>>> = images_per_chapter
        .into_par_iter()
        .enumerate()
        .filter_map(|(i, images_per_chapter)| {
            if images_per_chapter.is_empty() {
                trace!("Chapter {} has no images, skipping", i);
                return None;
            }

            let cover_path = images_per_chapter.into_iter().next().unwrap();
            let semaphore = Arc::clone(&semaphore);
            let book_start_chapters = Arc::clone(&book_start_chapters);
            trace!("Analyzing cover for chapter {}: {:?}", i, cover_path);

            Some(spawn(async move {
                let _permit = semaphore.acquire().await.map_err(|e| {
                    error!("Failed to acquire semaphore: {}", e);
                    Error::AsyncTaskError(format!("Failed to acquire semaphore: {}", e))
                })?;

                let is_color_cover = spawn_blocking(move || {
                    let cover_image = image::open(&cover_path)?;
                    // Inverted logic: return true if NOT grayscale (i.e., color cover)
                    Ok::<bool, Error>(!is_grayscale_with_threshold(
                        &cover_image,
                        GrayscaleStrategy::GridWithDownscale,
                        sensibility,
                    ))
                })
                .await
                .map_err(|e| Error::AsyncTaskError(e.to_string()))??;

                if is_color_cover {
                    debug!("Chapter {} identified as volume start", i);
                    book_start_chapters.lock().await.push(i);
                }

                Ok(())
            }))
        })
        .collect();

    // Wait for all tasks to complete
    for handle in handles {
        handle
            .await
            .map_err(|e| Error::AsyncTaskError(e.to_string()))??;
    }

    // Extract and sort results
    let result = Arc::try_unwrap(book_start_chapters)
        .map(|mutex| mutex.into_inner())
        .unwrap_or_else(|arc| (*arc.blocking_lock()).clone());

    let mut result = result;
    result.sort_unstable();
    info!("Identified {} volume start chapters", result.len());

    Ok(result)
}

/// Calculates chapter count per volume.
pub fn calculate_volume_sizes(
    mut book_start_chapters: Vec<usize>,
    total_chapters: usize,
) -> EResult<Vec<usize>> {
    info!(
        "Calculating volume sizes for {} total chapters",
        total_chapters
    );

    if book_start_chapters.is_empty() {
        log::warn!("No start chapters detected - returning empty volume sizes");
        return Ok(Vec::new());
    }

    debug!(
        "Removing first chapter {} (always a book start)",
        book_start_chapters[0]
    );
    book_start_chapters.remove(0);

    // Pre-allocate result vector
    let num_volumes = book_start_chapters.len() + 1;
    let mut book_chapters = Vec::with_capacity(num_volumes);

    let mut prev_chapter = 0;
    for chapter in book_start_chapters {
        let chapter_count = chapter - prev_chapter;
        debug!(
            "Volume with chapters {}-{}: {} chapters",
            prev_chapter,
            chapter - 1,
            chapter_count
        );
        book_chapters.push(chapter_count);
        prev_chapter = chapter;
    }

    let remaining = total_chapters - prev_chapter;
    debug!(
        "Final volume with chapters {}-{}: {} chapters",
        prev_chapter,
        total_chapters - 1,
        remaining
    );
    book_chapters.push(remaining);

    info!(
        "Calculated {} volumes with chapter counts: {:?}",
        book_chapters.len(),
        book_chapters
    );
    Ok(book_chapters)
}
