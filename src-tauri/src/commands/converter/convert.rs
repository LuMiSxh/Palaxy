//! Conversion module with parallel volume processing
//!
//! This module handles the main conversion process, converting bundled volumes
//! into CBZ or EPUB format with parallel processing and progress events.

use super::events::*;
use super::image::process_images_to_memory;
use crate::generator::Generator;
use crate::generator::cbz::Cbz;
use crate::generator::epub::EPub;
use crate::prelude::*;
use log::{debug, error, info, trace, warn};
use std::path::{Path, PathBuf};
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
    ) = {
        let state = state.lock().await;
        debug!(
            "Converting {} volumes to {:?} format",
            state.volume_sizes.len(),
            state.format
        );
        trace!(
            "Conversion parameters: target={:?}, create_directory={:?}, direction={:?}, image_format={:?}",
            state.target, state.create_directory, state.direction, state.image_format
        );

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
        )
    }; // Lock is released here

    debug!("Using target directory: {:?}", target);

    let target_directory_path = match create_directory {
        true => {
            let path = Path::new(&target).join(&name);
            debug!("Creating target directory: {:?}", path);
            if !path.exists() {
                trace!("Directory doesn't exist, creating it now");
                create_dir(&path).await.map_err(|e| {
                    error!("Failed to create directory {:?}: {}", path, e);
                    e
                })?;
            }
            Ok(path)
        }
        false => {
            let path = Path::new(&target);
            debug!("Using existing directory: {:?}", path);
            if !path.exists() {
                error!("Target directory does not exist: {:?}", path);
                Err(Error::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Target directory does not exist",
                )))
            } else {
                Ok(PathBuf::from(path))
            }
        }
    }?
    .to_str()
    .unwrap()
    .to_string();
    debug!("Target directory path: {}", target_directory_path);

    // Check if we have edited data, otherwise use normal data
    let pages = match edited_data {
        Some(e_data) => {
            if e_data.is_empty() {
                debug!("Using original data (edited data is empty)");
                data
            } else {
                debug!("Using edited data with {} chapter sets", e_data.len());
                e_data
            }
        }
        None => {
            debug!(
                "No edited data available, using original data with {} chapter sets",
                data.len()
            );
            data
        }
    };

    // Create a temporary directory for image conversions if needed
    let temp_dir = if image_format != ImageOutputFormat::None {
        let temp_path = Path::new(&target_directory_path).join(".palaxy_temp");
        debug!(
            "Creating temp directory for image conversions: {:?}",
            temp_path
        );
        if !temp_path.exists() {
            create_dir(&temp_path).await.map_err(|e| {
                error!("Failed to create temp directory {:?}: {}", temp_path, e);
                e
            })?;
        }
        Some(temp_path)
    } else {
        None
    };

    // Emit conversion start event
    let total_volumes = volume_sizes.len();
    if let Err(e) = emit_conversion_start(&app, total_volumes) {
        warn!("Failed to emit conversion start event: {}", e);
    }

    // Track results for final event
    let successful = Arc::new(StdMutex::new(0usize));
    let failed = Arc::new(StdMutex::new(0usize));

    // Clone temp_dir for cleanup later
    let temp_dir_cleanup = temp_dir.clone();

    info!(
        "Processing {} volumes in parallel using thread pool",
        total_volumes
    );

    // Pre-calculate cumulative chapter indices to avoid repeated summing
    let mut cumulative_indices = Vec::with_capacity(volume_sizes.len());
    let mut sum = 0;
    for &chapters in &volume_sizes {
        cumulative_indices.push(sum);
        sum += chapters;
    }

    // Use for_each instead of map/collect to emit events in real-time
    volume_sizes
        .iter() // <- NOTE: iter(), not par_iter()
        .enumerate()
        .for_each(|(i, &chapters)| {
            let j = cumulative_indices[i];
            // Use write! for better performance than format!
            let mut volume_name = String::with_capacity(name.len() + 10);
            use std::fmt::Write;
            write!(&mut volume_name, "{} | {}", name, i + 1).expect("String write cannot fail");

            debug!(
                "Volume {} ({}) will include {} chapters",
                i + 1,
                volume_name,
                chapters
            );

            // Clone the necessary data for this volume
            let volume_pages = pages[j..(j + chapters)].to_vec();

            // Emit volume start event
            if let Err(e) = emit_volume_start(&app, i, total_volumes, volume_name.clone()) {
                warn!("Failed to emit volume start event: {}", e);
            }

            // Emit status message for volume start
            if let Err(e) = emit_status_message(
                &app,
                StatusMessageType::VolumeStarted {
                    volume_index: i,
                    volume_name: volume_name.clone(),
                },
            ) {
                warn!("Failed to emit status message: {}", e);
            }

            debug!("Volume {}: starting conversion to {:?}", i + 1, format);

            // PERFORM CONVERSION
            let result = match format {
                FileFormat::Cbz => convert_cbz_volume(
                    i,
                    &volume_name,
                    &target_directory_path,
                    &volume_pages,
                    image_format,
                    &app,
                    total_volumes,
                ),
                FileFormat::Epub => convert_epub_volume(
                    i,
                    &volume_name,
                    &target_directory_path,
                    &volume_pages,
                    direction,
                    image_format,
                    &app,
                    total_volumes,
                ),
            };

            // Emit volume complete event
            match &result {
                Ok(_) => {
                    if let Ok(mut s) = successful.lock() {
                        *s += 1;
                    }
                    if let Err(e) = emit_volume_complete(
                        &app,
                        i,
                        total_volumes,
                        volume_name.clone(),
                        true,
                        None,
                    ) {
                        warn!("Failed to emit volume complete event: {}", e);
                    }

                    // Emit status message for successful volume completion
                    if let Err(e) = emit_status_message(
                        &app,
                        StatusMessageType::VolumeFinished {
                            volume_index: i,
                            volume_name: volume_name.clone(),
                            success: true,
                        },
                    ) {
                        warn!("Failed to emit status message: {}", e);
                    }
                }
                Err(err) => {
                    if let Ok(mut f) = failed.lock() {
                        *f += 1;
                    }
                    if let Err(e) = emit_volume_complete(
                        &app,
                        i,
                        total_volumes,
                        volume_name.clone(),
                        false,
                        Some(err.to_string()),
                    ) {
                        warn!("Failed to emit volume complete event: {}", e);
                    }

                    // Emit status message for failed volume completion
                    if let Err(e) = emit_status_message(
                        &app,
                        StatusMessageType::VolumeFinished {
                            volume_index: i,
                            volume_name: volume_name.clone(),
                            success: false,
                        },
                    ) {
                        warn!("Failed to emit status message: {}", e);
                    }
                }
            }

            // Log individual volume result but don't stop processing
            if let Err(e) = &result {
                error!("Volume {} failed: {}", i + 1, e);
            }
        });

    // Conversion is complete - all volumes processed in parallel
    let conversion_result = Ok(());

    // Clean up temporary directory if it was created
    if let Some(temp_dir) = temp_dir_cleanup {
        debug!("Cleaning up temporary directory: {:?}", temp_dir);
        if let Err(e) = tokio::fs::remove_dir_all(&temp_dir).await {
            warn!("Failed to remove temporary directory {:?}: {}", temp_dir, e);
        } else {
            trace!("Temporary directory removed successfully");
        }
    }

    let duration = start.elapsed().as_secs_f64();

    // Emit conversion complete event
    let successful_count = *successful.lock().unwrap();
    let failed_count = *failed.lock().unwrap();
    if let Err(e) = emit_conversion_complete(
        &app,
        total_volumes,
        successful_count,
        failed_count,
        duration,
    ) {
        warn!("Failed to emit conversion complete event: {}", e);
    }

    // Check if conversion had errors
    if let Err(e) = conversion_result {
        error!("Conversion process encountered errors: {}", e);
        return Err(e);
    }

    if failed_count > 0 {
        warn!(
            "Conversion completed with {} failures out of {} volumes",
            failed_count, total_volumes
        );
    }

    info!(
        "Conversion completed: {} successful, {} failed in {}s",
        successful_count, failed_count, duration
    );
    Ok(BaseResponse::default_duration(duration))
}

/// Convert a single volume to CBZ format
fn convert_cbz_volume(
    volume_index: usize,
    volume_name: &str,
    target_dir: &str,
    volume_pages: &[Vec<PathBuf>],
    image_format: ImageOutputFormat,
    app: &AppHandle,
    _total_volumes: usize,
) -> EResult<()> {
    let total_images: usize = volume_pages.iter().map(|c| c.len()).sum();

    // Use an Atomic counter for thread-safe progress tracking across the volume
    let current_image_atomic = Arc::new(AtomicUsize::new(0));
    let volume_name_arc = Arc::new(volume_name.to_string());
    let app_arc = Arc::new(app.clone());

    // Initialize Generator
    let mut generator = Cbz::new(target_dir, volume_name)?;

    // Define the progress callback
    let progress_callback = {
        let current_image = current_image_atomic.clone();
        let v_name = volume_name_arc.clone();
        let app_ref = app_arc.clone();

        move || {
            let curr = current_image.fetch_add(1, Ordering::Relaxed) + 1;

            // Emit progress frequently (every image or every 5th depending on preference)
            // Since AVIF is slow, emitting every image makes it feel responsive
            if let Err(e) = emit_status_message(
                &app_ref,
                StatusMessageType::PageAdded {
                    volume_index,
                    volume_name: v_name.to_string(),
                    page_number: curr,
                    total_pages: total_images,
                },
            ) {
                // Don't panic on event error, just log
                trace!("Failed to emit status: {}", e);
            }

            if curr % 5 == 0 || curr == total_images {
                let _ = emit_image_progress(
                    &app_ref,
                    volume_index,
                    v_name.to_string(),
                    curr,
                    total_images,
                );
            }
        }
    };

    trace!(
        "Volume {}: processing {} chapter sets",
        volume_index + 1,
        volume_pages.len()
    );

    for (chapter_idx, chapter_pages) in volume_pages.iter().enumerate() {
        let processed_pages =
            match process_images_to_memory(chapter_pages, image_format, Some(&progress_callback)) {
                Ok(pages) => pages,
                Err(e) => {
                    error!("Chapter {} failed: {}, trying passthrough", chapter_idx, e);
                    process_images_to_memory::<fn()>(
                        chapter_pages,
                        ImageOutputFormat::None, // Force original
                        None,                    // Don't double count progress
                    )?
                }
            };

        for page in processed_pages {
            generator.add_page_from_memory(&page.data, &page.extension)?;
        }
    }

    debug!(
        "Volume {}: setting metadata and saving CBZ",
        volume_index + 1
    );
    generator.set_metadata(volume_name, volume_index + 1)?;
    generator.save()?;

    info!("Volume {}: CBZ file saved successfully", volume_index + 1);
    Ok(())
}

/// Convert a single volume to EPUB format
fn convert_epub_volume(
    volume_index: usize,
    volume_name: &str,
    target_dir: &str,
    volume_pages: &[Vec<PathBuf>],
    direction: Direction,
    image_format: ImageOutputFormat,
    app: &AppHandle,
    _total_volumes: usize,
) -> EResult<()> {
    debug!(
        "Volume {}: creating EPUB file: {}",
        volume_index + 1,
        volume_name
    );

    if volume_pages.is_empty() || volume_pages[0].is_empty() {
        return Err(Error::Unsupported(
            "Cannot create EPUB without cover image".to_string(),
        ));
    }

    let total_images: usize = volume_pages.iter().map(|c| c.len()).sum();
    let current_image_atomic = Arc::new(AtomicUsize::new(0));
    let volume_name_arc = Arc::new(volume_name.to_string());
    let app_arc = Arc::new(app.clone());

    let mut generator = EPub::new(target_dir, volume_name)?;

    generator.set_cover(&volume_pages[0][0])?;

    generator.set_lang("en")?;
    generator.set_reading_direction(direction);
    generator.set_custom_metadata("title", volume_name)?;
    generator.set_custom_metadata("author", "Manga Bundler")?;

    // Progress callback (Same as CBZ)
    let progress_callback = {
        let current_image = current_image_atomic.clone();
        let v_name = volume_name_arc.clone();
        let app_ref = app_arc.clone();
        move || {
            let curr = current_image.fetch_add(1, Ordering::Relaxed) + 1;

            let _ = emit_status_message(
                &app_ref,
                StatusMessageType::PageAdded {
                    volume_index,
                    volume_name: v_name.to_string(),
                    page_number: curr,
                    total_pages: total_images,
                },
            );

            if curr % 5 == 0 || curr == total_images {
                let _ = emit_image_progress(
                    &app_ref,
                    volume_index,
                    v_name.to_string(),
                    curr,
                    total_images,
                );
            }
        }
    };

    for (chapter_idx, chapter_pages) in volume_pages.iter().enumerate() {
        // 1. PROCESS TO MEMORY (Parallel)
        let processed_pages =
            match process_images_to_memory(chapter_pages, image_format, Some(&progress_callback)) {
                Ok(pages) => pages,
                Err(e) => {
                    error!("Chapter {} failed: {}, trying passthrough", chapter_idx, e);
                    process_images_to_memory::<fn()>(chapter_pages, ImageOutputFormat::None, None)?
                }
            };

        for page in processed_pages {
            generator.add_page_from_memory(&page.data, &page.extension)?;
        }
    }

    generator.save()?;
    Ok(())
}
