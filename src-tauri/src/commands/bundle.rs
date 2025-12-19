//! Bundling module for organizing chapters into volumes
//!
//! This module handles the bundling of chapters into volumes based on
//! different strategies: manual, name-based, or image analysis.

use crate::prelude::*;
use log::{debug, info, trace, warn};
use scanner::{
    calculate_volume_sizes, determine_volume_start_chapters, sort_by_name_volume_chapter,
    sort_by_stem_number, sort_name_by_number, Collector,
};
use std::path::PathBuf;
use tauri::State;
use tokio::sync::Mutex;

/// Bundles chapters into volumes based on directory structure or image analysis.
///
/// This function collects all chapters, sorts them according to the bundling strategy,
/// and organizes them into volumes. The strategy depends on the `bundle_flag`:
/// - `Manual`: Basic sorting by numeric values in filenames
/// - `Name`: Intelligent sorting using volume-chapter naming conventions
/// - `Image`: Advanced sorting using grayscale detection to identify volume boundaries
///
/// # Arguments
/// * `sensibility` - Optional sensitivity parameter for image analysis (0-100)
/// * `state` - Application state containing conversion parameters
///
/// # Returns
/// * `EResult<CommBundle>` - Bundle information including chapter counts and volume distribution
#[tauri::command(async)]
#[specta::specta]
pub async fn conv_bundle(
    sensibility: Option<usize>,
    state: State<'_, Mutex<ConvState>>,
) -> EResult<CommBundle> {
    info!("Starting chapter bundling, sensibility: {:?}", sensibility);
    let start = std::time::Instant::now();

    let mut state = state.lock().await;
    trace!("Acquired state lock");

    debug!("Creating collector for source: {:?}", state.source);
    let mut collector = Collector::new(&state.source);

    // Collect all chapter and use a sorting algorithm based on BundleFlag
    debug!(
        "Collecting chapters using bundle flag: {:?}",
        state.bundle_flag
    );
    let chapters: Vec<PathBuf> = collector
        .collect_chapters(match state.bundle_flag {
            BundleFlag::Manual => {
                trace!("Using manual sorting strategy");
                Some(&sort_by_stem_number)
            }
            BundleFlag::Name => {
                trace!("Using name-based volume-chapter sorting strategy");
                Some(&sort_by_name_volume_chapter)
            }
            BundleFlag::Image => {
                trace!("Using numeric sorting strategy");
                Some(&sort_name_by_number)
            }
        })
        .await?;

    debug!("Collected {} chapters", chapters.len());

    // Collect all pages from the chapters
    debug!("Collecting pages from chapters");
    let pages: Vec<Vec<PathBuf>> = collector
        .collect_pages(chapters.clone(), Some(&sort_by_stem_number))
        .await?;

    let page_count: usize = pages.iter().map(|p| p.len()).sum();
    debug!(
        "Collected {} total pages across {} chapters",
        page_count,
        pages.len()
    );

    let total_chapters: usize = chapters.len();
    let mut total_volumes: usize = 0;
    let mut chapter_sizes: Vec<usize> = Vec::default();

    match state.bundle_flag {
        // For manual bundling, the user will have to manually input the remaining information.
        BundleFlag::Manual => {
            info!("Manual bundling selected - volume info will be provided by user");
        }
        // For automatic bundling by name
        BundleFlag::Name => {
            info!("Performing name-based bundling");
            let mut tmp = Vec::new();
            let mut extra = false;

            // Determine the start of each volume
            trace!("Determining volume boundaries based on naming conventions");
            for (i, chapter) in chapters.iter().enumerate() {
                let volume_number = chapter
                    .file_name()
                    .and_then(|name| name.to_str())
                    .and_then(|s| s.split('-').next())
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(0);

                trace!("Chapter {:?} has volume number {}", chapter, volume_number);

                if volume_number == 0 {
                    warn!("Found chapter with invalid volume number 0: {:?}", chapter);
                    extra = true;
                }

                if tmp.len() < volume_number {
                    debug!("Starting volume {} at chapter index {}", volume_number, i);
                    tmp.push(i);
                }
            }

            if extra {
                debug!("Adding supplementary volume for chapters with invalid volume numbers");
                tmp.push(0);
            }

            // Calculate the number of chapters per volume
            trace!("Calculating chapters per volume");
            let mut tmp2: Vec<usize> = Vec::new();
            for i in 0..tmp.len() {
                if i == tmp.len() - 1 {
                    let size = chapters.len() - tmp[i];
                    debug!("Volume {} has {} chapters", i + 1, size);
                    tmp2.push(size);
                } else {
                    let size = tmp[i + 1] - tmp[i];
                    debug!("Volume {} has {} chapters", i + 1, size);
                    tmp2.push(size);
                }
            }

            chapter_sizes = tmp2;
            total_volumes = tmp.len();
            info!("Name-based bundling resulted in {} volumes", total_volumes);
        }
        // Image-based bundling
        BundleFlag::Image => {
            info!("Performing image-based bundling with grayscale detection");
            let sensitivity = sensibility.map_or(0.75, |s| s as f64 / 100.0);
            debug!("Using grayscale sensitivity: {}", sensitivity);

            let volume_start_chapters =
                determine_volume_start_chapters(pages.clone(), sensitivity).await?;

            debug!("Detected {} volume boundaries", volume_start_chapters.len());
            for (i, &idx) in volume_start_chapters.iter().enumerate() {
                trace!("Volume {} starts at chapter index {}", i + 1, idx);
            }

            total_volumes = volume_start_chapters.len();
            chapter_sizes = calculate_volume_sizes(volume_start_chapters, total_chapters)?;

            debug!("Volume sizes: {:?}", chapter_sizes);
            info!("Image-based bundling resulted in {} volumes", total_volumes);
        }
    };

    // Set the new states
    debug!("Updating state with bundling results");
    state.volume_sizes = chapter_sizes.clone();
    state.data = pages;

    let duration = start.elapsed().as_secs_f64();
    info!("Bundling completed in {}s", duration);
    Ok(CommBundle {
        duration,
        comment: None,
        payload: Some(BundleResponse {
            total_chapters,
            total_volumes: if total_volumes > 0 {
                Some(total_volumes)
            } else {
                None
            },
            chapter_sizes: if !chapter_sizes.is_empty() {
                Some(chapter_sizes)
            } else {
                None
            },
        }),
    })
}
