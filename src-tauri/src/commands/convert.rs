//! Conversion module with parallel volume processing.

use crate::events::*;
use crate::prelude::*;
use log::{debug, error, info, trace, warn};
use packager::{process_images_to_memory, Cbz, EPub, Generator};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use tauri::{AppHandle, State};
use tokio::fs::create_dir;
use tokio::sync::Mutex;

/// Converts bundled volumes into the specified output format.
///
/// Processes volumes with concurrency control, generating output files in either
/// CBZ or EPUB format. Emits progress events to the frontend for real-time tracking.
///
/// # Arguments
/// * `state` - Application state containing conversion parameters
/// * `app` - Tauri application handle for event emission
///
/// # Returns
/// * `EResult<BaseResponse>` - Success response with execution duration
#[tauri::command(async)]
#[specta::specta]
pub async fn conv_convert(
    state: State<'_, Mutex<ConvState>>,
    app: AppHandle,
) -> EResult<BaseResponse> {
    info!("Starting volume conversion process");
    let start = std::time::Instant::now();

    // Extract all the necessary data while the lock is held
    let (
        name,
        target,
        create_directory,
        format,
        direction,
        image_format,
        volume_sizes,
        data,
        edited_data,
        hide_single_volume_number,
        volume_separator,
    ) = {
        let state = state.lock().await;
        (
            state.name.clone(),
            state.target.clone(),
            state.create_directory,
            state.format,
            state.direction,
            state.image_format,
            state.volume_sizes.clone(),
            state.data.clone(),
            state.edited_data.clone(),
            state.hide_single_volume_number,
            state.volume_separator.clone(),
        )
    };

    // Use edited data if available, otherwise use original data
    let data = edited_data.unwrap_or(data);

    debug!(
        "Conversion settings: format={:?}, direction={:?}, image_format={:?}",
        format, direction, image_format
    );
    debug!("Volume sizes: {:?}", volume_sizes);

    // Determine output path
    let output_path = if create_directory {
        let new_dir = target.join(&name);
        debug!("Creating new directory: {:?}", new_dir);
        if let Err(e) = create_dir(&new_dir).await {
            warn!("Failed to create directory {:?}: {}", new_dir, e);
        }
        new_dir
    } else {
        debug!("Using existing target directory: {:?}", target);
        target
    };

    let output_path_str = output_path
        .to_str()
        .ok_or_else(|| Error::InvalidPath(output_path.clone(), "Invalid UTF-8 path".into()))?;

    debug!("Output path: {}", output_path_str);

    // Calculate resource budget
    let budget = common::ResourceBudget::calculate();
    info!(
        "Using resource budget: {} concurrent volumes, rayon_chunk={}, memory_batch={}",
        budget.max_concurrent_volumes, budget.rayon_chunk_size, budget.memory_batch_size
    );

    // Emit conversion start event
    trace!("Emitting conversion start event");
    if let Err(e) = emit_conversion_start(&app, volume_sizes.len()) {
        warn!("Failed to emit conversion start event: {}", e);
    }

    // Prepare volumes (chunks of chapters)
    let mut volumes = Vec::new();
    let mut chapter_index = 0;

    for (vol_idx, &chapter_count) in volume_sizes.iter().enumerate() {
        let mut volume_data = Vec::new();
        for _ in 0..chapter_count {
            if chapter_index < data.len() {
                volume_data.push(data[chapter_index].clone());
                chapter_index += 1;
            }
        }
        volumes.push((vol_idx + 1, volume_data));
        trace!(
            "Prepared volume {} with {} chapters",
            vol_idx + 1,
            chapter_count
        );
    }

    info!("Processing {} volumes", volumes.len());

    // Shared counters for progress tracking
    let completed_volumes = Arc::new(AtomicUsize::new(0));
    let failed_volumes = Arc::new(StdMutex::new(Vec::new()));

    // Process volumes in parallel with controlled concurrency
    let semaphore = Arc::new(tokio::sync::Semaphore::new(budget.max_concurrent_volumes));

    let mut tasks = Vec::new();

    for (vol_idx, (volume_number, volume_chapters)) in volumes.into_iter().enumerate() {
        let sem = Arc::clone(&semaphore);
        let app_handle = app.clone();
        let name_clone = name.clone();
        let output_path_clone = output_path_str.to_string();
        let completed = Arc::clone(&completed_volumes);
        let failed = Arc::clone(&failed_volumes);
        let total_volumes = volume_sizes.len();
        let volume_name = if total_volumes == 1 && hide_single_volume_number {
            name_clone.clone()
        } else {
            format!("{}{}{}", name_clone, volume_separator, volume_number)
        };
        let rayon_chunk_size = budget.rayon_chunk_size;

        let task = tokio::spawn(async move {
            let _permit = sem.acquire().await.ok()?;

            debug!("Starting processing for volume {}", volume_number);

            // Emit volume start event
            if let Err(e) =
                emit_volume_start(&app_handle, vol_idx, total_volumes, volume_name.clone())
            {
                warn!("Failed to emit volume start event: {}", e);
            }

            // Flatten chapter images into a single list
            let all_images: Vec<PathBuf> = volume_chapters.into_iter().flatten().collect();
            let total_images = all_images.len();

            debug!("Volume {} has {} total images", volume_number, total_images);

            // Progress tracking
            let processed_images = Arc::new(AtomicUsize::new(0));
            let processed_clone = Arc::clone(&processed_images);
            let app_clone = app_handle.clone();
            let vol_name_clone = volume_name.clone();

            // Process images with progress callback
            let pages = process_images_to_memory(
                &all_images,
                image_format,
                Some(&|| {
                    let current = processed_clone.fetch_add(1, Ordering::SeqCst) + 1;
                    if let Err(e) = emit_image_progress(
                        &app_clone,
                        vol_idx,
                        vol_name_clone.clone(),
                        current,
                        total_images,
                    ) {
                        trace!("Failed to emit image progress: {}", e);
                    }
                }),
                Some(rayon_chunk_size),
            )
            .ok()?;

            debug!(
                "Processed {} images for volume {}",
                pages.len(),
                volume_number
            );

            debug!("Generating {:?} file: {}", format, volume_name);

            // Convert based on format
            let result = match format {
                FileFormat::Cbz => convert_cbz_volume(
                    &output_path_clone,
                    &volume_name,
                    &name_clone,
                    volume_number,
                    pages,
                ),
                FileFormat::Epub => convert_epub_volume(
                    &output_path_clone,
                    &volume_name,
                    &name_clone,
                    volume_number,
                    direction,
                    pages,
                ),
            };

            match result {
                Ok(_) => {
                    completed.fetch_add(1, Ordering::SeqCst);
                    info!("Successfully converted volume {}", volume_number);

                    // Emit volume complete event
                    if let Err(e) = emit_volume_complete(
                        &app_handle,
                        vol_idx,
                        total_volumes,
                        volume_name,
                        true,
                        None,
                    ) {
                        warn!("Failed to emit volume complete event: {}", e);
                    }

                    Some(())
                }
                Err(e) => {
                    error!("Failed to convert volume {}: {}", volume_number, e);
                    if let Ok(mut fails) = failed.lock() {
                        fails.push((volume_number, e.to_string()));
                    }

                    if let Err(err) = emit_volume_complete(
                        &app_handle,
                        vol_idx,
                        total_volumes,
                        volume_name,
                        false,
                        Some(e.to_string()),
                    ) {
                        warn!("Failed to emit volume complete event: {}", err);
                    }

                    None
                }
            }
        });

        tasks.push((vol_idx, volume_number, task));
    }

    debug!("Waiting for all volume conversion tasks to complete");
    for (vol_idx, volume_number, task) in tasks {
        match task.await {
            Ok(_) => {
                // Task finished normally (success or handled error inside)
            }
            Err(e) => {
                // Task panicked!
                let err_msg = if e.is_panic() {
                    "Worker thread crashed (Panic)".to_string()
                } else {
                    format!("Task failed: {}", e)
                };

                error!("Volume {} failed critically: {}", volume_number, err_msg);

                // Update failed count manually since the thread died
                if let Ok(mut fails) = failed_volumes.lock() {
                    fails.push((volume_number, err_msg.clone()));
                }

                // Construct volume name again for the event
                let total_volumes = volume_sizes.len();
                let vol_name = if total_volumes == 1 && hide_single_volume_number {
                    name.clone()
                } else {
                    format!("{}{}{}", name, volume_separator, volume_number)
                };

                // Notify frontend
                let _ = emit_volume_complete(
                    &app,
                    vol_idx,
                    total_volumes,
                    vol_name,
                    false,
                    Some(err_msg),
                );
            }
        }
    }

    let completed_count = completed_volumes.load(Ordering::SeqCst);
    let failed_count = failed_volumes.lock().unwrap().len();

    let duration = start.elapsed().as_secs_f64();

    info!(
        "Conversion complete: {} successful, {} failed in {:.2}s",
        completed_count, failed_count, duration
    );

    // Emit conversion complete event
    if let Err(e) = emit_conversion_complete(
        &app,
        volume_sizes.len(),
        completed_count,
        failed_count,
        duration,
    ) {
        warn!("Failed to emit conversion complete event: {}", e);
    }

    // Clean up temp directory if source was a ZIP file
    {
        let mut state = state.lock().await;
        if state.temp_dir.is_some() {
            debug!("Cleaning up temporary extraction directory");
            state.temp_dir = None; // Dropping Arc<TempDir> triggers cleanup
        }
    }

    Ok(BaseResponse::default_duration(duration))
}

fn convert_cbz_volume(
    output_path: &str,
    filename: &str,
    title: &str,
    volume_number: usize,
    pages: Vec<packager::ProcessedPage>,
) -> EResult<()> {
    debug!(
        "Creating CBZ: path={}, filename={}, volume={}",
        output_path, filename, volume_number
    );

    let mut cbz = Cbz::new(output_path, filename)?;

    debug!("Adding {} pages to CBZ", pages.len());
    for page in pages {
        cbz.add_page_from_memory(&page.data, &page.extension)?;
    }

    cbz.set_metadata(title, volume_number)?;

    debug!("Saving CBZ file");
    cbz.save()?;

    info!("CBZ volume {} created successfully", volume_number);
    Ok(())
}

fn convert_epub_volume(
    output_path: &str,
    filename: &str,
    title: &str,
    volume_number: usize,
    direction: Direction,
    pages: Vec<packager::ProcessedPage>,
) -> EResult<()> {
    debug!(
        "Creating EPUB: path={}, filename={}, volume={}, direction={:?}",
        output_path, filename, volume_number, direction
    );

    let mut epub = EPub::new(output_path, filename)?;

    epub.set_reading_direction(direction);
    epub.set_lang("en")?;

    debug!("Adding {} pages to EPUB", pages.len());
    for page in pages {
        epub.add_page_from_memory(&page.data, &page.extension)?;
    }

    epub.set_metadata(title, volume_number)?;

    debug!("Saving EPUB file");
    epub.save()?;

    info!("EPUB volume {} created successfully", volume_number);
    Ok(())
}
