//! Analysis module for source directory validation
//!
//! This module handles the analysis of source directories to validate structure,
//! naming conventions, file formats, and provide recommendations for bundling.

use crate::prelude::*;
use lazy_static::lazy_static;
use log::{debug, error, info, trace, warn};
use rayon::prelude::*;
use regex::Regex;
use scanner::Collector;
use std::path::PathBuf;
use tauri::State;
use tokio::sync::Mutex;

lazy_static! {
    /// Regular expression for analyzing chapter/volume naming patterns.
    /// Matches strings in format "digits-digits[.digits]" (e.g. "01-23" or "01-23.5").
    static ref REGEX_ANALYZE: Regex = Regex::new(r"\d+-\d+(\.\d+)?").unwrap();
}

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
            .filter(|&(_, &size)| size < avg_size / 3 || size > avg_size * 3)
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
