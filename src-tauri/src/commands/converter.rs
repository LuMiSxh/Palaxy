use crate::collector::Collector;
use crate::generator::cbz::Cbz;
use crate::generator::epub::EPub;
use crate::generator::Generator;
use crate::prelude::*;
use lazy_static::lazy_static;
use log::{debug, error, info, trace, warn};
use rayon::prelude::*;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::async_runtime::spawn;
use tauri::State;
use tokio::fs::create_dir;
use tokio::sync::{Mutex, Semaphore};

lazy_static! {
    /// Regular expression for analyzing chapter/volume naming patterns.
    /// Matches strings in format "digits-digits[.digits]" (e.g. "01-23" or "01-23.5").
    static ref REGEX_ANALYZE: Regex = Regex::new(r"\d+-\d+(\.\d+)?").unwrap();
}

// ConvState

/// Updates a specific field in the conversion state.
///
/// # Arguments
/// * `input` - Key-value pair specifying which state field is to update and its new value
/// * `state` - Application state containing conversion parameters
///
/// # Returns
/// * `EResult<BaseResponse>` - Success response with execution duration
#[tauri::command(async)]
#[specta::specta]
pub async fn conv_state_set(
    input: ConvStateKey,
    state: State<'_, Mutex<ConvState>>,
) -> EResult<BaseResponse> {
    info!("Setting conversion state: {:?}", input);
    let start = std::time::Instant::now();

    let mut state = state.lock().await;

    // Set the state based on the input
    match input {
        ConvStateKey::Name(value) => {
            debug!("Setting name to: {}", value);
            state.name = value;
        }
        ConvStateKey::Source(value) => {
            debug!("Setting source path to: {:?}", value);
            state.source = value;
        }
        ConvStateKey::Target(value) => {
            debug!("Setting target path to: {:?}", value);
            state.target = value;
        }
        ConvStateKey::BundleFlag(value) => {
            debug!("Setting bundle flag to: {:?}", value);
            state.bundle_flag = value;
        }
        ConvStateKey::Direction(value) => {
            debug!("Setting reading direction to: {:?}", value);
            state.direction = value;
        }
        ConvStateKey::Format(value) => {
            debug!("Setting output format to: {:?}", value);
            state.format = value;
        }
        ConvStateKey::CreateDirectory(value) => {
            debug!("Setting create directory flag to: {}", value);
            state.create_directory = value;
        }
        ConvStateKey::VolumeSizes(value) => {
            debug!("Setting volume sizes: {:?}", value);
            state.volume_sizes = value;
        }
        ConvStateKey::Data(value) => {
            debug!("Setting data with {} chapters", value.len());
            state.data = value;
        }
        ConvStateKey::EditedData(value) => {
            debug!(
                "Setting edited data: {} chapters",
                value.as_ref().map_or(0, |v| v.len())
            );
            state.edited_data = value;
        }
    }

    let duration = start.elapsed().as_secs_f64();
    trace!("State updated in {}s", duration);
    Ok(BaseResponse::default_duration(duration))
}

/// Retrieves the complete current conversion state.
///
/// # Arguments
/// * `state` - Application state containing conversion parameters
///
/// # Returns
/// * `EResult<BaseResponse<ConvState>>` - Success response containing the current state
#[tauri::command(async)]
#[specta::specta]
pub async fn conv_state_get(
    state: State<'_, Mutex<ConvState>>,
) -> EResult<BaseResponse<ConvState>> {
    info!("Getting conversion state");
    let start = std::time::Instant::now();

    let state = state.lock().await;
    trace!("Acquired state lock");

    let duration = start.elapsed().as_secs_f64();
    debug!("Retrieved conversion state in {}s", duration);
    Ok(BaseResponse {
        duration,
        comment: None,
        payload: Some(state.clone()),
    })
}

/// Resets the conversion state to default values.
///
/// # Arguments
/// * `state` - Application state containing conversion parameters
///
/// # Returns
/// * `EResult<BaseResponse>` - Success response with execution duration
#[tauri::command(async)]
#[specta::specta]
pub async fn conv_state_reset(state: State<'_, Mutex<ConvState>>) -> EResult<BaseResponse> {
    info!("Resetting conversion state");
    let start = std::time::Instant::now();

    let mut state = state.lock().await;
    trace!("Acquired state lock");
    state.reset();
    debug!("State reset to defaults");

    let duration = start.elapsed().as_secs_f64();
    debug!("Reset completed in {}s", duration);
    Ok(BaseResponse::default_duration(duration))
}

// -- PROCESSES --

/// Analyzes the source directory structure for conversion preparation.
///
/// Performs comprehensive validation and analysis of the source directory:
/// - Checks for proper directory structure and naming conventions
/// - Validates file formats and permissions
/// - Detects potential issues with file sizes, naming, and special characters
/// - Provides guidance on optimal bundling approaches
///
/// # Arguments
/// * `state` - Application state containing conversion parameters
///
/// # Returns
/// * `EResult<CommAnalyzeMeta>` - Analysis results with positive findings, warnings, and errors
#[tauri::command(async)]
#[specta::specta]
pub async fn conv_analyze(state: State<'_, Mutex<ConvState>>) -> EResult<CommAnalyzeMeta> {
    info!("Starting source directory analysis");
    let start = std::time::Instant::now();

    let state = state.lock().await;
    debug!("Analyzing source path: {:?}", state.source);

    fn has_perms(path: &PathBuf) -> bool {
        path.metadata()
            .map(|meta| !meta.permissions().readonly())
            .unwrap_or(false)
    }

    let mut negative = Vec::new();
    let mut positive = Vec::new();
    let mut warning = Vec::new();
    let mut flag = BundleFlag::Image;
    let mut collector = Collector::new(&state.source);
    debug!("Created collector for path: {:?}", state.source);

    // Handle errors for the collection
    trace!("Collecting chapters");
    let chapters = match collector.collect_chapters(None).await {
        Ok(chapters) => {
            debug!("Found {} chapters", chapters.len());
            chapters
        }
        Err(e) => {
            error!("Failed to collect chapters: {}", e);
            negative.push(e.to_string());
            return Ok(CommAnalyzeMeta {
                duration: start.elapsed().as_secs_f64(),
                comment: None,
                payload: Some(AnalyzeResponse {
                    negative,
                    positive,
                    warning,
                    flag,
                }),
            });
        }
    };

    // Handle errors for the collection
    trace!("Collecting pages");
    let mut pages = match chapters.is_empty() {
        true => {
            debug!("No chapters found, page collection skipped");
            Vec::new()
        }
        false => match collector.collect_pages(chapters.clone(), None).await {
            Ok(pages) => {
                let page_count: usize = pages.iter().map(|p| p.len()).sum();
                debug!(
                    "Collected {} pages across {} chapters",
                    page_count,
                    pages.len()
                );
                pages.concat()
            }
            Err(e) => {
                error!("Failed to collect pages: {}", e);
                negative.push(e.to_string());
                return Ok(CommAnalyzeMeta {
                    duration: start.elapsed().as_secs_f64(),
                    comment: None,
                    payload: Some(AnalyzeResponse {
                        negative,
                        positive,
                        warning,
                        flag,
                    }),
                });
            }
        },
    };

    pages.retain(|path| path.is_file());
    trace!("Filtered to {} actual files", pages.len());

    if chapters.is_empty() {
        warn!("No subdirectories found in source path");
        negative.push(
            "No subdirectories found in the base path. Ensure they exist with images inside."
                .to_string(),
        );

        return Ok(CommAnalyzeMeta {
            duration: start.elapsed().as_secs_f64(),
            comment: None,
            payload: Some(AnalyzeResponse {
                negative,
                positive,
                warning,
                flag,
            }),
        });
    }

    if pages.is_empty() && !chapters.is_empty() {
        warn!("Subdirectories exist but contain no files");
        negative.push(
            "Subdirectories contain no files. Verify that chapter images are placed inside."
                .to_string(),
        );

        return Ok(CommAnalyzeMeta {
            duration: start.elapsed().as_secs_f64(),
            comment: None,
            payload: Some(AnalyzeResponse {
                negative,
                positive,
                warning,
                flag,
            }),
        });
    }

    trace!("Checking for numeric identifiers in directory names");
    let dir_lacks_numeric = Collector::check_path(&chapters, |path| {
        path.file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .chars()
            .any(char::is_numeric)
    })?;

    let numeric_errors: Vec<String> = dir_lacks_numeric
        .par_iter()
        .map(|dir| {
            debug!(
                "Directory lacks numerical identifiers: {:?}",
                dir.file_name().unwrap()
            );
            format!(
                "Directory {:?} lacks numerical identifiers. Remove them for faster bundling.",
                dir.file_name().unwrap()
            )
        })
        .collect();

    negative.extend(numeric_errors);

    // Image Format Validation
    trace!("Validating image formats");
    let unsupported_formats = Collector::check_path(&pages, |path| {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some(ext) => matches!(ext.to_lowercase().as_str(), "jpg" | "jpeg" | "png" | "webp"),
            None => false, // Files without extensions are unsupported
        }
    })?;

    if !unsupported_formats.is_empty() {
        warn!(
            "Found {} unsupported file formats",
            unsupported_formats.len()
        );
        let format_examples: Vec<String> = unsupported_formats
            .par_iter()
            .take(3)
            .filter_map(|path| path.file_name().map(|n| n.to_string_lossy().to_string()))
            .collect();

        let example_msg = if !format_examples.is_empty() {
            format!(" Examples: {}", format_examples.join(", "))
        } else {
            String::new()
        };

        negative.push(format!(
            "Found {} files with unsupported formats. Only JPG, PNG, and WebP are fully supported.{}",
            unsupported_formats.len(),
            example_msg
        ));
    }

    // File Size Consistency Check
    trace!("Checking file size consistency");
    let file_sizes: Vec<u64> = pages
        .par_iter()
        .filter_map(|p| p.metadata().ok().map(|m| m.len()))
        .collect();

    if !file_sizes.is_empty() {
        let avg_size = file_sizes.par_iter().sum::<u64>() / file_sizes.len() as u64;
        debug!("Average image file size: {} bytes", avg_size);
        let outliers: Vec<_> = file_sizes
            .par_iter()
            .enumerate()
            .filter(|(_, &size)| size < avg_size / 3 || size > avg_size * 3)
            .collect();

        if !outliers.is_empty() && outliers.len() < file_sizes.len() / 10 {
            warn!("Detected {} files with unusual sizes", outliers.len());
            warning.push(format!("Detected {} files with unusual sizes. These might be cover pages, blank pages, or corrupted images.", outliers.len()));
        }
    }

    // File Count Consistency
    trace!("Checking chapter file count consistency");
    if !chapters.is_empty() {
        let chapter_pages = collector.collect_pages(chapters.clone(), None).await;
        if let Ok(pages_vec) = chapter_pages {
            let chapter_file_counts: Vec<usize> = pages_vec.par_iter().map(|p| p.len()).collect();

            if !chapter_file_counts.is_empty() {
                let avg_count =
                    chapter_file_counts.iter().sum::<usize>() / chapter_file_counts.len();
                debug!("Average pages per chapter: {}", avg_count);
                let outliers = chapter_file_counts
                    .par_iter()
                    .filter(|&&count| count < avg_count / 2 || count > avg_count * 2)
                    .count();

                if outliers > 0 {
                    warn!(
                        "Found {} chapters with significant page count differences",
                        outliers
                    );
                    warning.push(format!("Found {} chapters with significantly different page counts. This may indicate missing pages or incorrectly organized content.", outliers));
                }
            }
        }
    }

    // Path Length Warning
    trace!("Checking path lengths");
    let long_paths = pages
        .par_iter()
        .filter(|p| p.to_string_lossy().len() > 240)
        .count();

    if long_paths > 0 {
        warn!("Found {} very long paths", long_paths);
        warning.push(format!(
            "Found {} paths that are very long. This may cause issues on some operating systems.",
            long_paths
        ));
    }

    // Special Character Check
    trace!("Checking for special characters in paths");
    let special_chars = chapters
        .par_iter()
        .filter(|path| {
            path.to_string_lossy().contains(|c: char| {
                !(c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' || c == '.' || c == '/')
            })
        })
        .count();

    if special_chars > 0 {
        warn!(
            "Found {} directories with special characters",
            special_chars
        );
        warning.push(
            "Some directories contain special characters which may cause issues during processing."
                .to_string(),
        );
    }

    trace!("Checking file and directory permissions");
    let permission_errors: Vec<String> = chapters
        .par_iter()
        .filter_map(|chapter| {
            if !has_perms(chapter) {
                warn!("Directory lacks permissions: {:?}", chapter);
                Some(format!(
                    "Directory {:?} lacks write permissions. Required for full functionality.",
                    chapter.file_name().unwrap()
                ))
            } else {
                None
            }
        })
        .collect();

    negative.extend(permission_errors);

    pages.iter().for_each(|page| {
        if !has_perms(page) {
            warn!("File lacks permissions: {:?}", page);
            negative.push(format!(
                "File '{:?}' lacks write permissions. Required for full functionality.",
                page.file_name().unwrap()
            ));
        }
    });

    trace!("Checking directory naming conventions");
    let dir_lacks_naming = Collector::check_path(&chapters, |path| {
        REGEX_ANALYZE.is_match(path.file_name().unwrap().to_str().unwrap())
    })?;

    if !dir_lacks_naming.is_empty() {
        warn!(
            "Directory naming convention not followed for {} directories",
            dir_lacks_naming.len()
        );
        warning.push("Subdirectory naming convention not followed; use 'VOLUME-CHAPTER' (e.g., '002-032') for faster bundling.".to_string());
    }

    if dir_lacks_numeric.is_empty() && dir_lacks_naming.is_empty() {
        info!("Directories correctly named and numbered");
        positive.push("Directories correctly named and numbered. Automatic bundling will proceed with the fastest algorithm.".to_string());
        flag = BundleFlag::Name;
    } else {
        warn!("Will use fallback bundling mechanisms");
        warning.push("Automatic bundling will use fallback mechanisms, potentially slowing the process and increasing error risk.".to_string());
    }

    trace!("Checking file naming consistency");
    let file_lack_numeric = Collector::check_path(&pages, |path| {
        path.file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .replace(".", "")
            .chars()
            .all(char::is_numeric)
    })?;

    file_lack_numeric.iter().for_each(|file| {
        debug!(
            "File lacks numerical naming: {:?}",
            file.file_name().unwrap()
        );
        negative.push(format!(
            "File {:?} lacks numerical naming. Required for effective sorting and bundling.",
            file.file_name().unwrap()
        ));
    });

    // Check if all images are consistently named
    trace!("Validating image naming consistency");
    let consistently_named = pages
        .par_iter()
        .map(|path| path.file_stem().and_then(|s| s.to_str()))
        .all(|stem| {
            stem.map(|s| {
                let numeric_part = s.chars().filter(|c| c.is_ascii_digit()).collect::<String>();
                !numeric_part.is_empty() && numeric_part.parse::<u32>().is_ok()
            })
            .unwrap_or(false)
        });

    if consistently_named {
        debug!("All image files are consistently named");
        positive.push("All image files are consistently named with numeric identifiers. This ensures correct page ordering.".to_string());
    }

    // Check for reasonable total image count
    if !pages.is_empty() && pages.len() <= 10000 {
        debug!("Image count {} is within reasonable limits", pages.len());
        positive.push(format!(
            "Total of {} images found. Amount is within reasonable processing limits.",
            pages.len()
        ));
    } else if pages.len() > 10000 {
        warn!("Large number of images detected: {}", pages.len());
        warning.push(format!(
            "Large number of images detected ({}). Processing may take a long time.",
            pages.len()
        ));
    }

    // Check if all images are in the same format
    trace!("Checking image format consistency");
    let image_formats = pages
        .par_iter()
        .filter_map(|p| p.extension().and_then(|e| e.to_str()))
        .map(|e| e.to_lowercase())
        .collect::<std::collections::HashSet<_>>();

    if image_formats.len() == 1 {
        debug!(
            "All images use the same format: {}",
            image_formats
                .iter()
                .next()
                .unwrap_or(&"unknown".to_string())
        );
        positive.push(format!(
            "All images use the same format ({}). This ensures consistent quality and processing.",
            image_formats
                .iter()
                .next()
                .unwrap_or(&"unknown".to_string())
        ));
    } else {
        debug!("Multiple image formats detected: {:?}", image_formats);
    }

    let duration = start.elapsed().as_secs_f64();
    info!(
        "Analysis completed in {}s with {} positive, {} warnings, {} negative findings",
        duration,
        positive.len(),
        warning.len(),
        negative.len()
    );
    Ok(CommAnalyzeMeta {
        duration,
        comment: None,
        payload: Some(AnalyzeResponse {
            negative,
            positive,
            warning,
            flag,
        }),
    })
}

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
                Some(&Collector::sort_by_stem_number)
            }
            BundleFlag::Name => {
                trace!("Using name-based volume-chapter sorting strategy");
                Some(&Collector::sort_by_name_volume_chapter)
            }
            BundleFlag::Image => {
                trace!("Using numeric sorting strategy");
                Some(&Collector::sort_name_by_number)
            }
        })
        .await?;

    debug!("Collected {} chapters", chapters.len());

    // Collect all pages from the chapters
    debug!("Collecting pages from chapters");
    let pages: Vec<Vec<PathBuf>> = collector
        .collect_pages(chapters.clone(), Some(&Collector::sort_by_stem_number))
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

            let volume_start_chapters = collector
                .determine_volume_start_chapters(pages.clone(), sensitivity)
                .await?;

            debug!("Detected {} volume boundaries", volume_start_chapters.len());
            for (i, &idx) in volume_start_chapters.iter().enumerate() {
                trace!("Volume {} starts at chapter index {}", i + 1, idx);
            }

            total_volumes = volume_start_chapters.len();
            chapter_sizes =
                collector.calculate_volume_sizes(volume_start_chapters, total_chapters)?;

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

/// Converts bundled volumes into the specified output format.
///
/// Processes all volumes in parallel, generating output files in either CBZ or EPUB format
/// according to the configuration. Creates directories as needed and applies appropriate
/// metadata to the generated files.
///
/// # Arguments
/// * `state` - Application state containing conversion parameters
///
/// # Returns
/// * `EResult<BaseResponse>` - Success response with execution duration
#[tauri::command(async)]
#[specta::specta]
pub async fn conv_convert(state: State<'_, Mutex<ConvState>>) -> EResult<BaseResponse> {
    info!("Starting volume conversion process");
    let start = std::time::Instant::now();

    // Extract all the necessary data while the lock is held
    let (name, target, create_directory, format, direction, volume_sizes, data, edited_data) = {
        let state = state.lock().await;
        debug!(
            "Converting {} volumes to {:?} format",
            state.volume_sizes.len(),
            state.format
        );
        trace!(
            "Conversion parameters: target={:?}, create_directory={:?}, direction={:?}",
            state.target,
            state.create_directory,
            state.direction
        );

        (
            state.name.clone(),
            state.target.clone(),
            state.create_directory,
            state.format,
            state.direction,
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

    // check if we have edited data, otherwise use normal data
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

    // Create a semaphore to limit concurrent conversions
    let max_concurrent = num_cpus::get().min(10);
    info!(
        "Processing with {} concurrent conversion threads",
        max_concurrent
    );
    let semaphore = Arc::new(Semaphore::new(max_concurrent));

    // Create a vector of tasks, each processing one volume
    let mut tasks = Vec::new();
    debug!("Preparing {} volumes for conversion", volume_sizes.len());

    for (i, &chapters) in volume_sizes.iter().enumerate() {
        let j: usize = volume_sizes[0..i].par_iter().sum();
        let volume_name = format!("{} | {}", name.clone(), i + 1);
        let target_dir = target_directory_path.clone();
        let format_clone = format;
        let direction_clone = direction;
        let semaphore_clone = Arc::clone(&semaphore);

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

        // Spawn a task for each volume to process them in parallel
        let task = spawn(async move {
            trace!("Volume {}: waiting for available thread", i + 1);
            let _permit = semaphore_clone.acquire().await.map_err(|e| {
                error!("Volume {}: failed to acquire semaphore: {}", i + 1, e);
                Error::AsyncTaskError(format!("Failed to acquire semaphore: {}", e))
            })?;
            debug!(
                "Volume {}: starting conversion to {:?}",
                i + 1,
                format_clone
            );

            match format_clone {
                FileFormat::Cbz => {
                    debug!("Volume {}: creating CBZ file: {}", i + 1, volume_name);
                    let mut generator = match Cbz::new(&target_dir, &volume_name) {
                        Ok(gen) => gen,
                        Err(e) => {
                            error!("Volume {}: failed to create CBZ generator: {}", i + 1, e);
                            return Err(e);
                        }
                    };

                    trace!(
                        "Volume {}: adding {} chapter sets to CBZ",
                        i + 1,
                        volume_pages.len()
                    );
                    for (chapter_idx, chapter_pages) in volume_pages.iter().enumerate() {
                        trace!(
                            "Volume {}, Chapter {}: adding {} pages",
                            i + 1,
                            chapter_idx + 1,
                            chapter_pages.len()
                        );
                        for page in chapter_pages {
                            if let Err(e) = generator.add_page(page).await {
                                error!(
                                    "Volume {}, Chapter {}: failed to add page {:?}: {}",
                                    i + 1,
                                    chapter_idx + 1,
                                    page,
                                    e
                                );
                                return Err(e);
                            }
                        }
                    }

                    debug!("Volume {}: setting metadata and saving CBZ", i + 1);
                    if let Err(e) = generator.set_metadata(&volume_name, i + 1).await {
                        error!("Volume {}: failed to set metadata: {}", i + 1, e);
                        return Err(e);
                    }

                    if let Err(e) = generator.save().await {
                        error!("Volume {}: failed to save CBZ: {}", i + 1, e);
                        return Err(e);
                    }
                    info!("Volume {}: CBZ file saved successfully", i + 1);
                }
                FileFormat::Epub => {
                    debug!("Volume {}: creating EPUB file: {}", i + 1, volume_name);
                    if volume_pages.is_empty() || volume_pages[0].is_empty() {
                        error!("Volume {}: cannot create EPUB without cover image", i + 1);
                        return Err(Error::Unsupported(
                            "Cannot create EPUB without cover image".to_string(),
                        ));
                    }

                    let mut generator = match EPub::new(&target_dir, &volume_name) {
                        Ok(gen) => gen,
                        Err(e) => {
                            error!("Volume {}: failed to create EPUB generator: {}", i + 1, e);
                            return Err(e);
                        }
                    };

                    debug!("Volume {}: setting EPUB cover and properties", i + 1);
                    if let Err(e) = generator.set_cover(&volume_pages[0][0]) {
                        error!("Volume {}: failed to set cover image: {}", i + 1, e);
                        return Err(e);
                    }

                    generator.set_lang("en")?;
                    generator.set_reading_direction(direction_clone);

                    trace!("Volume {}: setting EPUB metadata", i + 1);
                    if let Err(e) = generator.set_custom_metadata("title", &volume_name) {
                        error!("Volume {}: failed to set title metadata: {}", i + 1, e);
                        return Err(e);
                    }

                    if let Err(e) = generator.set_custom_metadata("author", "Manga Bundler") {
                        error!("Volume {}: failed to set author metadata: {}", i + 1, e);
                        return Err(e);
                    }

                    debug!(
                        "Volume {}: adding {} chapters to EPUB",
                        i + 1,
                        volume_pages.len()
                    );
                    for (chapter_idx, chapter_pages) in volume_pages.iter().enumerate() {
                        trace!(
                            "Volume {}: adding chapter {} with {} pages",
                            i + 1,
                            chapter_idx + 1,
                            chapter_pages.len()
                        );
                        if let Err(e) = generator.add_chapter(chapter_idx + 1, chapter_pages).await
                        {
                            error!(
                                "Volume {}: failed to add chapter {}: {}",
                                i + 1,
                                chapter_idx + 1,
                                e
                            );
                            return Err(e);
                        }
                    }

                    debug!("Volume {}: saving EPUB file", i + 1);
                    if let Err(e) = generator.save().await {
                        error!("Volume {}: failed to save EPUB: {}", i + 1, e);
                        return Err(e);
                    }
                    info!("Volume {}: EPUB file saved successfully", i + 1);
                }
            }

            Ok(())
        });

        tasks.push(task);
    }

    info!("Waiting for {} conversion tasks to complete", tasks.len());
    for (i, task) in tasks.into_iter().enumerate() {
        match task.await {
            Ok(result) => match result {
                Ok(_) => debug!("Volume {} conversion completed successfully", i + 1),
                Err(e) => {
                    error!("Volume {} conversion failed: {}", i + 1, e);
                    return Err(e);
                }
            },
            Err(e) => {
                error!("Volume {} task panicked: {}", i + 1, e);
                return Err(Error::AsyncTaskError(e.to_string()));
            }
        }
    }

    let duration = start.elapsed().as_secs_f64();
    info!(
        "All volume conversions completed successfully in {}s",
        duration
    );
    Ok(BaseResponse::default_duration(duration))
}
