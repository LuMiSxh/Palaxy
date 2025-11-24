//! Conversion module with parallel volume processing
//!
//! This module handles the main conversion process, converting bundled volumes
//! into CBZ or EPUB format with parallel processing and progress events.

use super::events::*;
use super::image::convert_image;
use crate::generator::Generator;
use crate::generator::cbz::Cbz;
use crate::generator::epub::EPub;
use crate::prelude::*;
use log::{debug, error, info, trace, warn};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, State};
use tokio::fs::create_dir;
use tokio::sync::{Mutex, Semaphore};
use tokio::task::{LocalSet, spawn_blocking, spawn_local};

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

    // Create a semaphore to limit concurrent conversions
    let max_concurrent = num_cpus::get().min(10);
    info!(
        "Processing with {} concurrent conversion threads",
        max_concurrent
    );
    let semaphore = Arc::new(Semaphore::new(max_concurrent));

    // Clone temp_dir before moving into async block so we can clean it up later
    let temp_dir_cleanup = temp_dir.clone();

    // Track results for final event
    let successful = Arc::new(Mutex::new(0usize));
    let failed = Arc::new(Mutex::new(0usize));

    // Clone values needed after spawn_blocking
    let successful_final = Arc::clone(&successful);
    let failed_final = Arc::clone(&failed);
    let app_final = app.clone();

    // Spawn blocking task that creates its own LocalSet
    let conversion_result = spawn_blocking(move || {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async move {
            let local = LocalSet::new();
            local
                .run_until(async move {
                    // Create a vector of tasks, each processing one volume
                    let mut tasks = Vec::new();
                    let total_volumes = volume_sizes.len();
                    debug!("Preparing {} volumes for conversion", total_volumes);

                    for (i, &chapters) in volume_sizes.iter().enumerate() {
                        let j: usize = volume_sizes[0..i].iter().sum();
                        let volume_name = format!("{} | {}", name.clone(), i + 1);
                        let target_dir = target_directory_path.clone();
                        let format_clone = format;
                        let direction_clone = direction;
                        let image_format_clone = image_format;
                        let temp_dir_clone = temp_dir.clone();
                        let semaphore_clone = Arc::clone(&semaphore);
                        let app_clone = app.clone();
                        let successful_clone = Arc::clone(&successful);
                        let failed_clone = Arc::clone(&failed);

                        debug!(
                            "Volume {} ({}) will include {} chapters",
                            i + 1,
                            volume_name,
                            chapters
                        );

                        // Clone the necessary data for this volume
                        let volume_pages = pages[j..(j + chapters)].to_vec();
                        trace!(
                            "Volume {} has {} chapter sets with total {} pages",
                            i + 1,
                            volume_pages.len(),
                            volume_pages.iter().map(|c| c.len()).sum::<usize>()
                        );

                        // Spawn a local task
                        let task = spawn_local(async move {
                            trace!("Volume {}: waiting for available thread", i + 1);
                            let _permit = semaphore_clone.acquire().await.map_err(|e| {
                                error!("Volume {}: failed to acquire semaphore: {}", i + 1, e);
                                Error::AsyncTaskError(format!("Failed to acquire semaphore: {}", e))
                            })?;

                            // Emit volume start event
                            if let Err(e) =
                                emit_volume_start(&app_clone, i, total_volumes, volume_name.clone())
                            {
                                warn!("Failed to emit volume start event: {}", e);
                            }

                            // Emit status message for volume start
                            if let Err(e) = emit_status_message(
                                &app_clone,
                                StatusMessageType::VolumeStarted {
                                    volume_index: i,
                                    volume_name: volume_name.clone(),
                                },
                            ) {
                                warn!("Failed to emit status message: {}", e);
                            }

                            debug!(
                                "Volume {}: starting conversion to {:?}",
                                i + 1,
                                format_clone
                            );

                            // Perform conversion
                            let result = match format_clone {
                                FileFormat::Cbz => {
                                    convert_cbz_volume(
                                        i,
                                        &volume_name,
                                        &target_dir,
                                        &volume_pages,
                                        image_format_clone,
                                        temp_dir_clone.as_ref(),
                                        &app_clone,
                                        total_volumes,
                                    )
                                    .await
                                }
                                FileFormat::Epub => {
                                    convert_epub_volume(
                                        i,
                                        &volume_name,
                                        &target_dir,
                                        &volume_pages,
                                        direction_clone,
                                        image_format_clone,
                                        temp_dir_clone.as_ref(),
                                        &app_clone,
                                        total_volumes,
                                    )
                                    .await
                                }
                            };

                            // Emit volume complete event
                            match &result {
                                Ok(_) => {
                                    let mut s = successful_clone.lock().await;
                                    *s += 1;
                                    if let Err(e) = emit_volume_complete(
                                        &app_clone,
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
                                        &app_clone,
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
                                    let mut f = failed_clone.lock().await;
                                    *f += 1;
                                    if let Err(e) = emit_volume_complete(
                                        &app_clone,
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
                                        &app_clone,
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

                            result
                        });

                        tasks.push(task);
                    }

                    info!("Waiting for {} conversion tasks to complete", tasks.len());
                    for (i, task) in tasks.into_iter().enumerate() {
                        match task.await {
                            Ok(result) => match result {
                                Ok(_) => {
                                    debug!("Volume {} conversion completed successfully", i + 1)
                                }
                                Err(e) => {
                                    error!("Volume {} conversion failed: {}", i + 1, e);
                                    // Don't return error, let other volumes complete
                                }
                            },
                            Err(e) => {
                                error!("Volume {} task panicked: {}", i + 1, e);
                            }
                        }
                    }

                    Ok::<(), Error>(())
                })
                .await
        })
    })
    .await
    .map_err(|e| {
        error!("Blocking task panicked: {}", e);
        Error::AsyncTaskError(format!("Blocking task panicked: {}", e))
    });

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
    let successful_count = *successful_final.lock().await;
    let failed_count = *failed_final.lock().await;
    if let Err(e) = emit_conversion_complete(
        &app_final,
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
async fn convert_cbz_volume(
    volume_index: usize,
    volume_name: &str,
    target_dir: &str,
    volume_pages: &[Vec<PathBuf>],
    image_format: ImageOutputFormat,
    temp_dir: Option<&PathBuf>,
    app: &AppHandle,
    _total_volumes: usize,
) -> EResult<()> {
    debug!(
        "Volume {}: creating CBZ file: {}",
        volume_index + 1,
        volume_name
    );
    let mut generator = Cbz::new(target_dir, volume_name)?;

    let total_images: usize = volume_pages.iter().map(|c| c.len()).sum();
    let mut current_image = 0;

    trace!(
        "Volume {}: adding {} chapter sets to CBZ",
        volume_index + 1,
        volume_pages.len()
    );

    for (chapter_idx, chapter_pages) in volume_pages.iter().enumerate() {
        trace!(
            "Volume {}, Chapter {}: adding {} pages",
            volume_index + 1,
            chapter_idx + 1,
            chapter_pages.len()
        );

        for page in chapter_pages {
            let page_to_add = if image_format != ImageOutputFormat::None {
                if let Some(temp_dir) = temp_dir {
                    match convert_image(page, temp_dir, image_format).await {
                        Ok(converted_path) => converted_path,
                        Err(e) => {
                            error!(
                                "Volume {}, Chapter {}: failed to convert image to {:?}: {}",
                                volume_index + 1,
                                chapter_idx + 1,
                                image_format,
                                e
                            );
                            page.clone()
                        }
                    }
                } else {
                    page.clone()
                }
            } else {
                page.clone()
            };

            generator.add_page(&page_to_add).await?;

            current_image += 1;

            // Emit status message for page added
            if let Err(e) = emit_status_message(
                app,
                StatusMessageType::PageAdded {
                    volume_index,
                    volume_name: volume_name.to_string(),
                    page_number: current_image,
                    total_pages: total_images,
                },
            ) {
                warn!("Failed to emit status message: {}", e);
            }

            // Emit progress every 5 images
            if current_image % 5 == 0 || current_image == total_images {
                if let Err(e) = emit_image_progress(
                    app,
                    volume_index,
                    volume_name.to_string(),
                    current_image,
                    total_images,
                ) {
                    warn!("Failed to emit image progress event: {}", e);
                }
            }
        }
    }

    debug!(
        "Volume {}: setting metadata and saving CBZ",
        volume_index + 1
    );
    generator
        .set_metadata(volume_name, volume_index + 1)
        .await?;
    generator.save().await?;
    info!("Volume {}: CBZ file saved successfully", volume_index + 1);

    Ok(())
}

/// Convert a single volume to EPUB format
async fn convert_epub_volume(
    volume_index: usize,
    volume_name: &str,
    target_dir: &str,
    volume_pages: &[Vec<PathBuf>],
    direction: Direction,
    image_format: ImageOutputFormat,
    temp_dir: Option<&PathBuf>,
    app: &AppHandle,
    _total_volumes: usize,
) -> EResult<()> {
    debug!(
        "Volume {}: creating EPUB file: {}",
        volume_index + 1,
        volume_name
    );

    if volume_pages.is_empty() || volume_pages[0].is_empty() {
        error!(
            "Volume {}: cannot create EPUB without cover image",
            volume_index + 1
        );
        return Err(Error::Unsupported(
            "Cannot create EPUB without cover image".to_string(),
        ));
    }

    let mut generator = EPub::new(target_dir, volume_name)?;

    debug!(
        "Volume {}: setting EPUB cover and properties",
        volume_index + 1
    );
    generator.set_cover(&volume_pages[0][0])?;
    generator.set_lang("en")?;
    generator.set_reading_direction(direction);

    trace!("Volume {}: setting EPUB metadata", volume_index + 1);
    generator.set_custom_metadata("title", volume_name)?;
    generator.set_custom_metadata("author", "Manga Bundler")?;

    let total_images: usize = volume_pages.iter().map(|c| c.len()).sum();
    let mut current_image = 0;

    debug!(
        "Volume {}: adding {} chapters to EPUB",
        volume_index + 1,
        volume_pages.len()
    );

    for (chapter_idx, chapter_pages) in volume_pages.iter().enumerate() {
        trace!(
            "Volume {}: adding chapter {} with {} pages",
            volume_index + 1,
            chapter_idx + 1,
            chapter_pages.len()
        );

        // Convert chapter pages if needed
        let chapter_pages_to_add: Vec<PathBuf> = if image_format != ImageOutputFormat::None {
            if let Some(temp_dir) = temp_dir {
                let mut converted_pages = Vec::new();
                for page in chapter_pages {
                    match convert_image(page, temp_dir, image_format).await {
                        Ok(converted_path) => converted_pages.push(converted_path),
                        Err(e) => {
                            error!(
                                "Volume {}, Chapter {}: failed to convert image to {:?}: {}",
                                volume_index + 1,
                                chapter_idx + 1,
                                image_format,
                                e
                            );
                            converted_pages.push(page.clone());
                        }
                    }

                    current_image += 1;

                    // Emit status message for page added
                    if let Err(e) = emit_status_message(
                        app,
                        StatusMessageType::PageAdded {
                            volume_index,
                            volume_name: volume_name.to_string(),
                            page_number: current_image,
                            total_pages: total_images,
                        },
                    ) {
                        warn!("Failed to emit status message: {}", e);
                    }

                    // Emit progress every 5 images
                    if current_image % 5 == 0 || current_image == total_images {
                        if let Err(e) = emit_image_progress(
                            app,
                            volume_index,
                            volume_name.to_string(),
                            current_image,
                            total_images,
                        ) {
                            warn!("Failed to emit image progress event: {}", e);
                        }
                    }
                }
                converted_pages
            } else {
                chapter_pages.clone()
            }
        } else {
            for _ in chapter_pages {
                current_image += 1;

                // Emit status message for page added
                if let Err(e) = emit_status_message(
                    app,
                    StatusMessageType::PageAdded {
                        volume_index,
                        volume_name: volume_name.to_string(),
                        page_number: current_image,
                        total_pages: total_images,
                    },
                ) {
                    warn!("Failed to emit status message: {}", e);
                }

                if current_image % 5 == 0 || current_image == total_images {
                    if let Err(e) = emit_image_progress(
                        app,
                        volume_index,
                        volume_name.to_string(),
                        current_image,
                        total_images,
                    ) {
                        warn!("Failed to emit image progress event: {}", e);
                    }
                }
            }
            chapter_pages.clone()
        };

        generator
            .add_chapter(chapter_idx + 1, &chapter_pages_to_add)
            .await?;
    }

    debug!("Volume {}: saving EPUB file", volume_index + 1);
    generator.save().await?;
    info!("Volume {}: EPUB file saved successfully", volume_index + 1);

    Ok(())
}
