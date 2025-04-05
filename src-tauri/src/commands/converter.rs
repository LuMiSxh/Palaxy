use crate::collector::Collector;
use crate::generator::cbz::Cbz;
use crate::generator::epub::EPub;
use crate::generator::Generator;
use crate::prelude::*;
use lazy_static::lazy_static;
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
/// * `input` - Key-value pair specifying which state field to update and its new value
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
    let start = std::time::Instant::now();

    let mut state = state.lock().await;

    // Set the state based on the input
    match input {
        ConvStateKey::Name(value) => state.name = value,
        ConvStateKey::Source(value) => state.source = value,
        ConvStateKey::Target(value) => state.target = value,
        ConvStateKey::BundleFlag(value) => state.bundle_flag = value,
        ConvStateKey::Direction(value) => state.direction = value,
        ConvStateKey::Format(value) => state.format = value,
        ConvStateKey::CreateDirectory(value) => state.create_directory = value,
        ConvStateKey::VolumeSizes(value) => state.volume_sizes = value,
        ConvStateKey::Data(value) => state.data = value,
        ConvStateKey::EditedData(value) => state.edited_data = value,
    }

    Ok(BaseResponse::default_duration(
        start.elapsed().as_secs_f64(),
    ))
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
    let start = std::time::Instant::now();

    let state = state.lock().await;

    Ok(BaseResponse {
        duration: start.elapsed().as_secs_f64(),
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
    let start = std::time::Instant::now();

    let mut state = state.lock().await;
    state.reset();

    Ok(BaseResponse::default_duration(
        start.elapsed().as_secs_f64(),
    ))
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
    let start = std::time::Instant::now();

    let state = state.lock().await;

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

    // Handle errors for the collection
    let chapters = match collector.collect_chapters(None).await {
        Ok(chapters) => chapters,
        Err(e) => {
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
    let mut pages = match chapters.is_empty() {
        true => Vec::new(),
        false => match collector.collect_pages(chapters.clone(), None).await {
            Ok(pages) => pages.concat(),
            Err(e) => {
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

    if chapters.is_empty() {
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
            format!(
                "Directory {:?} lacks numerical identifiers. Remove them for faster bundling.",
                dir.file_name().unwrap()
            )
        })
        .collect();

    negative.extend(numeric_errors);

    // Image Format Validation
    let unsupported_formats = Collector::check_path(&pages, |path| {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some(ext) => matches!(ext.to_lowercase().as_str(), "jpg" | "jpeg" | "png" | "webp"),
            None => false, // Files without extensions are unsupported
        }
    })?;

    if !unsupported_formats.is_empty() {
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
    let file_sizes: Vec<u64> = pages
        .par_iter()
        .filter_map(|p| p.metadata().ok().map(|m| m.len()))
        .collect();

    if !file_sizes.is_empty() {
        let avg_size = file_sizes.par_iter().sum::<u64>() / file_sizes.len() as u64;
        let outliers: Vec<_> = file_sizes
            .par_iter()
            .enumerate()
            .filter(|(_, &size)| size < avg_size / 3 || size > avg_size * 3)
            .collect();

        if !outliers.is_empty() && outliers.len() < file_sizes.len() / 10 {
            warning.push(format!("Detected {} files with unusual sizes. These might be cover pages, blank pages, or corrupted images.", outliers.len()));
        }
    }

    // File Count Consistency
    if !chapters.is_empty() {
        let chapter_pages = collector.collect_pages(chapters.clone(), None).await;
        if let Ok(pages_vec) = chapter_pages {
            let chapter_file_counts: Vec<usize> = pages_vec.par_iter().map(|p| p.len()).collect();

            if !chapter_file_counts.is_empty() {
                let avg_count =
                    chapter_file_counts.iter().sum::<usize>() / chapter_file_counts.len();
                let outliers = chapter_file_counts
                    .par_iter()
                    .filter(|&&count| count < avg_count / 2 || count > avg_count * 2)
                    .count();

                if outliers > 0 {
                    warning.push(format!("Found {} chapters with significantly different page counts. This may indicate missing pages or incorrectly organized content.", outliers));
                }
            }
        }
    }

    // Path Length Warning
    let long_paths = pages
        .par_iter()
        .filter(|p| p.to_string_lossy().len() > 240)
        .count();

    if long_paths > 0 {
        warning.push(format!(
            "Found {} paths that are very long. This may cause issues on some operating systems.",
            long_paths
        ));
    }

    // Special Character Check
    let special_chars = chapters
        .par_iter()
        .filter(|path| {
            path.to_string_lossy().contains(|c: char| {
                !(c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' || c == '.' || c == '/')
            })
        })
        .count();

    if special_chars > 0 {
        warning.push(
            "Some directories contain special characters which may cause issues during processing."
                .to_string(),
        );
    }

    let permission_errors: Vec<String> = chapters
        .par_iter()
        .filter_map(|chapter| {
            if !has_perms(chapter) {
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
            negative.push(format!(
                "File '{:?}' lacks write permissions. Required for full functionality.",
                page.file_name().unwrap()
            ));
        }
    });

    let dir_lacks_naming = Collector::check_path(&chapters, |path| {
        REGEX_ANALYZE.is_match(path.file_name().unwrap().to_str().unwrap())
    })?;

    if !dir_lacks_naming.is_empty() {
        warning.push("Subdirectory naming convention not followed; use 'VOLUME-CHAPTER' (e.g., '002-032') for faster bundling.".to_string());
    }

    if dir_lacks_numeric.is_empty() && dir_lacks_naming.is_empty() {
        positive.push("Directories correctly named and numbered. Automatic bundling will proceed with the fastest algorithm.".to_string());
        flag = BundleFlag::Name;
    } else {
        warning.push("Automatic bundling will use fallback mechanisms, potentially slowing the process and increasing error risk.".to_string());
    }

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
        negative.push(format!(
            "File {:?} lacks numerical naming. Required for effective sorting and bundling.",
            file.file_name().unwrap()
        ));
    });

    // Check if all images are consistently named
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
        positive.push("All image files are consistently named with numeric identifiers. This ensures correct page ordering.".to_string());
    }

    // Check for reasonable total image count
    if !pages.is_empty() && pages.len() <= 10000 {
        positive.push(format!(
            "Total of {} images found. Amount is within reasonable processing limits.",
            pages.len()
        ));
    } else if pages.len() > 10000 {
        warning.push(format!(
            "Large number of images detected ({}). Processing may take a long time.",
            pages.len()
        ));
    }

    // Check if all images are in the same format
    let image_formats = pages
        .par_iter()
        .filter_map(|p| p.extension().and_then(|e| e.to_str()))
        .map(|e| e.to_lowercase())
        .collect::<std::collections::HashSet<_>>();

    if image_formats.len() == 1 {
        positive.push(format!(
            "All images use the same format ({}). This ensures consistent quality and processing.",
            image_formats
                .iter()
                .next()
                .unwrap_or(&"unknown".to_string())
        ));
    }

    Ok(CommAnalyzeMeta {
        duration: start.elapsed().as_secs_f64(),
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
    let start = std::time::Instant::now();

    let mut state = state.lock().await;
    let mut collector = Collector::new(&state.source);

    // Collect all chapter and use a sorting algorithm based on BundleFlag
    let chapters: Vec<PathBuf> = collector
        .collect_chapters(match state.bundle_flag {
            BundleFlag::Manual => Some(&Collector::sort_by_stem_number),
            BundleFlag::Name => Some(&Collector::sort_by_name_volume_chapter),
            BundleFlag::Image => Some(&Collector::sort_name_by_number),
        })
        .await?;

    // Collect all pages from the chapters
    let pages: Vec<Vec<PathBuf>> = collector
        .collect_pages(chapters.clone(), Some(&Collector::sort_by_stem_number))
        .await?;

    let total_chapters: usize = chapters.len();
    let mut total_volumes: usize = 0;
    let mut chapter_sizes: Vec<usize> = Vec::default();

    match state.bundle_flag {
        // For manual bundling, the user will have to manually input the remaining information.
        // This will be done in the frontend.
        BundleFlag::Manual => {}
        // For automatic bundling by name,
        // the program will use the naming convention
        // to determine the volumes and chapters.
        BundleFlag::Name => {
            let mut tmp = Vec::new();
            let mut extra = false;

            // Determine the start of each volume
            for (i, chapter) in chapters.iter().enumerate() {
                let volume_number = chapter
                    .file_name()
                    .and_then(|name| name.to_str())
                    .and_then(|s| s.split('-').next())
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(0);

                if volume_number == 0 {
                    extra = true;
                }

                if tmp.len() < volume_number {
                    tmp.push(i);
                }
            }

            if extra {
                tmp.push(0);
            }

            // Calculate the number of chapters per volume based on the start chapters of each volume
            let mut tmp2: Vec<usize> = Vec::new();
            for i in 0..tmp.len() {
                if i == tmp.len() - 1 {
                    tmp2.push(chapters.len() - tmp[i]);
                } else {
                    tmp2.push(tmp[i + 1] - tmp[i]);
                }
            }

            chapter_sizes = tmp2;
            total_volumes = tmp.len();
        }
        // The image version uses the grayscale detection algorithm to determine the start of each volume.
        // This is done by checking the first image of each chapter.
        BundleFlag::Image => {
            let volume_start_chapters = collector
                .determine_volume_start_chapters(
                    pages.clone(),
                    sensibility.map_or(0.75, |s| s as f64 / 100.0),
                )
                .await?;

            total_volumes = volume_start_chapters.len();
            chapter_sizes =
                collector.calculate_volume_sizes(volume_start_chapters, total_chapters)?;
        }
    };

    // Set the new states
    state.volume_sizes = chapter_sizes.clone();
    state.data = pages;

    Ok(CommBundle {
        duration: start.elapsed().as_secs_f64(),
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
    let start = std::time::Instant::now();

    // Extract all the necessary data while the lock is held
    let (name, target, create_directory, format, direction, volume_sizes, data, edited_data) = {
        let state = state.lock().await;
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

    let target_directory_path = match create_directory {
        true => {
            let path = Path::new(&target).join(&name);
            if !path.exists() {
                create_dir(&path).await?;
            }
            Ok(path)
        }
        false => {
            let path = Path::new(&target);
            if !path.exists() {
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

    // check if we have edited data, otherwise use normal data
    let pages = match edited_data {
        Some(e_data) => {
            if e_data.is_empty() {
                data
            } else {
                e_data
            }
        }
        None => data,
    };

    // Create a semaphore to limit concurrent conversions
    // Use available CPU cores or a reasonable fixed limit
    let max_concurrent = num_cpus::get().min(10); // Use at most 10 threads or available cores, whichever is smaller
    let semaphore = Arc::new(Semaphore::new(max_concurrent));

    // Create a vector of tasks, each processing one volume
    let mut tasks = Vec::new();

    for (i, &chapters) in volume_sizes.iter().enumerate() {
        let j: usize = volume_sizes[0..i].par_iter().sum();
        let volume_name = format!("{} | {}", name.clone(), i + 1);
        let target_dir = target_directory_path.clone();
        let format_clone = format;
        let direction_clone = direction;
        let semaphore_clone = Arc::clone(&semaphore);

        // Clone the necessary data for this volume
        let volume_pages = pages[j..(j + chapters)].to_vec();

        // Spawn a task for each volume to process them in parallel
        let task = spawn(async move {
            // Acquire a permit from the semaphore before starting the conversion
            let _permit = semaphore_clone.acquire().await.map_err(|e| {
                Error::AsyncTaskError(format!("Failed to acquire semaphore: {}", e))
            })?;

            match format_clone {
                FileFormat::Cbz => {
                    // Create a new CBZ generator
                    let mut generator = Cbz::new(&target_dir, &volume_name)?;

                    // Add all pages from the chapters
                    for (_, chapter_pages) in volume_pages.iter().enumerate() {
                        for page in chapter_pages {
                            generator.add_page(page).await?;
                        }
                    }

                    // Set metadata and save
                    generator.set_metadata(&volume_name, i + 1).await?;
                    generator.save().await?;
                }
                FileFormat::Epub => {
                    // Make sure we have pages for the cover
                    if volume_pages.is_empty() || volume_pages[0].is_empty() {
                        return Err(Error::Unsupported(
                            "Cannot create EPUB without cover image".to_string(),
                        ));
                    }

                    // Create a new EPUB generator
                    let mut generator = EPub::new(&target_dir, &volume_name)?;

                    // Set cover, language, and reading direction
                    generator.set_cover(&volume_pages[0][0])?;
                    generator.set_lang("en")?;
                    generator.set_reading_direction(direction_clone);

                    // Set metadata
                    generator.set_custom_metadata("title", &volume_name)?;
                    generator.set_custom_metadata("author", "Manga Bundler")?;

                    // Add chapters
                    for (chapter_idx, chapter_pages) in volume_pages.iter().enumerate() {
                        generator
                            .add_chapter(chapter_idx + 1, chapter_pages)
                            .await?;
                    }

                    // Save the EPUB
                    generator.save().await?;
                }
            }

            Ok(())
        });

        tasks.push(task);
    }

    // Wait for all tasks to complete
    for task in tasks {
        task.await
            .map_err(|e| Error::AsyncTaskError(e.to_string()))??;
    }

    Ok(BaseResponse::default_duration(
        start.elapsed().as_secs_f64(),
    ))
}
