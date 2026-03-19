//! Bundling module for organizing chapters into volumes.
//!
//! Handles bundling based on different strategies: manual, name-based,
//! image analysis, or flattening into a single volume.

use crate::prelude::*;
use log::{debug, info, trace, warn};
use scanner::{
    calculate_volume_sizes, determine_volume_start_chapters, extract_volume_chapter, Collector,
};
use std::collections::BTreeMap;
use tauri::State;
use tokio::sync::Mutex;

/// Bundles chapters into volumes based on directory structure or image analysis.
///
/// Uses the new `Collector::discover()` API that supports ZIP input, flat structures,
/// and arbitrary nesting depth. Sorts using `natural_sort` by default.
///
/// # Arguments
/// * `sensibility` - Optional sensitivity parameter for image analysis (0-100)
/// * `state` - Application state containing conversion parameters
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

    // Use new collector API — discovers structure automatically
    debug!("Creating collector for source: {:?}", state.source);
    let collector = Collector::from_path(&state.source)?;
    let discover = collector.discover()?;

    let chapters = discover.chapters;
    let total_chapters = chapters.len();

    debug!("Discovered {} chapters, flat={}", total_chapters, discover.is_flat);

    if total_chapters == 0 {
        warn!("No chapters found");
        state.volume_sizes = Vec::new();
        state.data = Vec::new();
        return Ok(CommBundle {
            duration: start.elapsed().as_secs_f64(),
            comment: None,
            payload: Some(BundleResponse {
                total_chapters: 0,
                total_volumes: None,
                chapter_sizes: None,
            }),
        });
    }

    // Handle flatten option: merge everything into one volume
    if state.flatten {
        info!("Flatten enabled — merging all chapters into one volume");
        state.volume_sizes = vec![total_chapters];
        state.data = chapters;

        let duration = start.elapsed().as_secs_f64();
        return Ok(CommBundle {
            duration,
            comment: None,
            payload: Some(BundleResponse {
                total_chapters,
                total_volumes: Some(1),
                chapter_sizes: Some(vec![total_chapters]),
            }),
        });
    }

    // Handle flat input: single chapter = single volume
    if discover.is_flat {
        info!("Flat structure — treating as single volume");
        state.volume_sizes = vec![1];
        state.data = chapters;

        let duration = start.elapsed().as_secs_f64();
        return Ok(CommBundle {
            duration,
            comment: None,
            payload: Some(BundleResponse {
                total_chapters: 1,
                total_volumes: Some(1),
                chapter_sizes: Some(vec![1]),
            }),
        });
    }

    let mut total_volumes: usize = 0;
    let mut chapter_sizes: Vec<usize> = Vec::new();

    match state.bundle_flag {
        BundleFlag::Manual => {
            info!("Manual bundling selected — volume info will be provided by user");
        }

        BundleFlag::Name => {
            info!("Performing name-based bundling");

            // Use extract_volume_chapter to group chapters by volume number
            let mut volume_groups: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
            for (i, chapter_pages) in chapters.iter().enumerate() {
                let dir_path = chapter_pages
                    .first()
                    .and_then(|p| p.parent())
                    .unwrap_or(std::path::Path::new(""));

                let (vol, _ch) = extract_volume_chapter(dir_path);
                let vol_num = vol.unwrap_or(0);
                trace!("Chapter {} -> volume {}", i, vol_num);
                volume_groups.entry(vol_num).or_default().push(i);
            }

            // Convert groups to chapter_sizes (in volume order)
            for (vol_num, chapter_indices) in &volume_groups {
                let count = chapter_indices.len();
                debug!("Volume {}: {} chapters", vol_num, count);
                chapter_sizes.push(count);
            }

            total_volumes = volume_groups.len();
            info!("Name-based bundling resulted in {} volumes", total_volumes);
        }

        BundleFlag::Image => {
            info!("Performing image-based bundling with grayscale detection");
            let sensitivity = sensibility.map_or(0.75, |s| s as f64 / 100.0);
            debug!("Using grayscale sensitivity: {}", sensitivity);

            let volume_start_chapters =
                determine_volume_start_chapters(chapters.clone(), sensitivity).await?;

            debug!("Detected {} volume boundaries", volume_start_chapters.len());
            total_volumes = volume_start_chapters.len();
            chapter_sizes = calculate_volume_sizes(volume_start_chapters, total_chapters)?;

            debug!("Volume sizes: {:?}", chapter_sizes);
            info!("Image-based bundling resulted in {} volumes", total_volumes);
        }
    }

    // Update state
    debug!("Updating state with bundling results");
    state.volume_sizes = chapter_sizes.clone();
    state.data = chapters;

    let duration = start.elapsed().as_secs_f64();
    info!("Bundling completed in {:.2}s", duration);
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
