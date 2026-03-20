//! Analysis module for source directory/ZIP validation.
//!
//! Performs comprehensive validation of the input source (directory or ZIP)
//! including structure detection, naming conventions, file formats, and
//! provides recommendations for bundling strategy.

use crate::prelude::*;
use common::utils::{is_image_file, SUPPORTED_IMAGE_EXTENSIONS};
use log::{debug, info, trace, warn};
use rayon::prelude::*;
use scanner::Collector;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

/// Holds the results from a single validator.
struct ValidationFindings {
    negative: Vec<String>,
    positive: Vec<String>,
    warning: Vec<String>,
}

impl ValidationFindings {
    fn new() -> Self {
        Self {
            negative: Vec::new(),
            positive: Vec::new(),
            warning: Vec::new(),
        }
    }
}

/// Analyzes the source structure for conversion preparation.
///
/// Supports both directories and ZIP files. Performs validation of structure,
/// naming conventions, file formats, permissions, and provides bundling recommendations.
#[tauri::command(async)]
#[specta::specta]
pub async fn conv_analyze(state: State<'_, Mutex<ConvState>>) -> EResult<CommAnalyzeMeta> {
    info!("Starting source analysis");
    let start = std::time::Instant::now();

    let mut state = state.lock().await;
    debug!("Analyzing source path: {:?}", state.source);

    // Step 1: Create collector (handles ZIP extraction transparently)
    let mut collector = match Collector::from_path(&state.source) {
        Ok(c) => c,
        Err(e) => {
            return Ok(error_response(start, e.to_string()));
        }
    };

    // Step 2: Discover structure
    let discover = match collector.discover() {
        Ok(d) => d,
        Err(e) => {
            return Ok(error_response(start, e.to_string()));
        }
    };

    // Step 3: Store temp dir handle in state if ZIP was extracted
    if let Some(temp) = collector.take_temp_dir() {
        let temp_path = temp.path().to_path_buf();
        state.temp_dir = Some(Arc::new(temp));
        state.source = temp_path;
    }
    state.is_flat = discover.is_flat;
    state.skipped_files = discover.skipped_files.clone();

    // Step 4: Check for empty input
    if discover.chapters.is_empty() {
        return Ok(error_response(
            start,
            "No image files found in the source. Ensure the directory or ZIP contains supported image files.".into(),
        ));
    }

    let all_pages: Vec<&PathBuf> = discover.chapters.iter().flat_map(|c| c.iter()).collect();

    // Collect chapter directory paths for validators that need them
    let chapter_dirs: Vec<PathBuf> = discover
        .chapters
        .iter()
        .filter_map(|chapter| chapter.first().and_then(|p| p.parent()).map(|p| p.to_path_buf()))
        .collect();

    // Step 5: Run all validators in parallel using nested rayon::join
    let (left, right) = rayon::join(
        || {
            rayon::join(
                || {
                    rayon::join(
                        || validate_image_formats(&all_pages),
                        || validate_file_sizes(&all_pages),
                    )
                },
                || {
                    rayon::join(
                        || validate_chapter_consistency(&discover.chapters),
                        || validate_path_lengths(&all_pages),
                    )
                },
            )
        },
        || {
            rayon::join(
                || {
                    rayon::join(
                        || validate_special_chars(&chapter_dirs),
                        || validate_permissions(&chapter_dirs, &all_pages),
                    )
                },
                || {
                    rayon::join(
                        || {
                            rayon::join(
                                || validate_file_naming(&all_pages),
                                || validate_image_count(&all_pages),
                            )
                        },
                        || {
                            rayon::join(
                                || validate_format_uniformity(&all_pages),
                                || validate_skipped_files(&discover.skipped_files),
                            )
                        },
                    )
                },
            )
        },
    );

    // Destructure nested tuples
    let ((format_findings, size_findings), (consistency_findings, path_findings)) = left;
    let (
        (special_findings, perm_findings),
        ((file_naming_findings, count_findings), (uniformity_findings, skipped_findings)),
    ) = right;

    // Naming convention check (also determines bundle flag)
    // Only run if not flat (flat structures have no chapter directories to check)
    let (naming_result, flag) = if discover.is_flat {
        debug!("Flat structure detected, skipping directory naming validation");
        (ValidationFindings::new(), BundleFlag::Manual)
    } else {
        validate_naming_convention(&chapter_dirs)
    };

    // Step 6: Aggregate results
    let mut negative = Vec::new();
    let mut positive = Vec::new();
    let mut warning = Vec::new();

    // Add flat structure info
    if discover.is_flat {
        positive.push("Flat directory detected: all images at root level.".into());
    }

    for findings in [
        &format_findings,
        &size_findings,
        &consistency_findings,
        &path_findings,
        &special_findings,
        &perm_findings,
        &naming_result,
        &file_naming_findings,
        &count_findings,
        &uniformity_findings,
        &skipped_findings,
    ] {
        negative.extend_from_slice(&findings.negative);
        positive.extend_from_slice(&findings.positive);
        warning.extend_from_slice(&findings.warning);
    }

    let duration = start.elapsed().as_secs_f64();
    info!(
        "Analysis completed in {:.2}s: {} positive, {} warnings, {} negative",
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

/// Creates an error response with a single negative finding.
fn error_response(start: std::time::Instant, message: String) -> CommAnalyzeMeta {
    CommAnalyzeMeta {
        duration: start.elapsed().as_secs_f64(),
        comment: None,
        payload: Some(AnalyzeResponse {
            negative: vec![message],
            positive: Vec::new(),
            warning: Vec::new(),
            flag: BundleFlag::Image,
        }),
    }
}

// --- Validator Functions ---

fn validate_image_formats(pages: &[&PathBuf]) -> ValidationFindings {
    trace!("Validating image formats");
    let mut findings = ValidationFindings::new();

    let unsupported: Vec<_> = pages
        .par_iter()
        .filter(|path| !is_image_file(path))
        .collect();

    if !unsupported.is_empty() {
        warn!("Found {} unsupported file formats", unsupported.len());
        let examples: Vec<String> = unsupported
            .iter()
            .take(3)
            .filter_map(|path| path.file_name().map(|n| n.to_string_lossy().to_string()))
            .collect();

        let example_msg = if !examples.is_empty() {
            format!(" Examples: {}", examples.join(", "))
        } else {
            String::new()
        };

        findings.negative.push(format!(
            "Found {} files with unsupported formats. Supported: {}.{}",
            unsupported.len(),
            SUPPORTED_IMAGE_EXTENSIONS.join(", "),
            example_msg
        ));
    }

    findings
}

fn validate_file_sizes(pages: &[&PathBuf]) -> ValidationFindings {
    trace!("Checking file size consistency");
    let mut findings = ValidationFindings::new();

    let file_sizes: Vec<u64> = pages
        .par_iter()
        .filter_map(|p| p.metadata().ok().map(|m| m.len()))
        .collect();

    if file_sizes.is_empty() {
        return findings;
    }

    let avg_size = file_sizes.par_iter().sum::<u64>() / file_sizes.len() as u64;
    let outliers: Vec<_> = file_sizes
        .par_iter()
        .filter(|&&size| size < avg_size / 3 || size > avg_size * 3)
        .collect();

    if !outliers.is_empty() && outliers.len() < file_sizes.len() / 10 {
        findings.warning.push(format!(
            "Detected {} files with unusual sizes. These might be cover pages, blank pages, or corrupted images.",
            outliers.len()
        ));
    }

    findings
}

fn validate_chapter_consistency(chapters: &[Vec<PathBuf>]) -> ValidationFindings {
    trace!("Checking chapter file count consistency");
    let mut findings = ValidationFindings::new();

    if chapters.len() <= 1 {
        return findings;
    }

    let counts: Vec<usize> = chapters.par_iter().map(|c| c.len()).collect();
    let avg = counts.iter().sum::<usize>() / counts.len();

    if avg == 0 {
        return findings;
    }

    let outliers = counts
        .par_iter()
        .filter(|&&count| count < avg / 2 || count > avg * 2)
        .count();

    if outliers > 0 {
        findings.warning.push(format!(
            "Found {} chapters with significantly different page counts. This may indicate missing pages or incorrectly organized content.",
            outliers
        ));
    }

    findings
}

fn validate_path_lengths(pages: &[&PathBuf]) -> ValidationFindings {
    trace!("Checking path lengths");
    let mut findings = ValidationFindings::new();

    let long_paths = pages
        .par_iter()
        .filter(|p| p.to_string_lossy().len() > 240)
        .count();

    if long_paths > 0 {
        findings.warning.push(format!(
            "Found {} paths that are very long. This may cause issues on some operating systems.",
            long_paths
        ));
    }

    findings
}

fn validate_special_chars(chapter_dirs: &[PathBuf]) -> ValidationFindings {
    trace!("Checking for special characters in paths");
    let mut findings = ValidationFindings::new();

    let special_count = chapter_dirs
        .par_iter()
        .filter(|path| {
            path.to_string_lossy().contains(|c: char| {
                !(c.is_alphanumeric()
                    || c == '-'
                    || c == '_'
                    || c == ' '
                    || c == '.'
                    || c == '/'
                    || c == '\\')
            })
        })
        .count();

    if special_count > 0 {
        findings.warning.push(
            "Some directories contain special characters which may cause issues during processing."
                .into(),
        );
    }

    findings
}

fn validate_permissions(chapter_dirs: &[PathBuf], pages: &[&PathBuf]) -> ValidationFindings {
    trace!("Checking file and directory permissions");
    let mut findings = ValidationFindings::new();

    fn has_perms(path: &std::path::Path) -> bool {
        path.metadata()
            .map(|meta| !meta.permissions().readonly())
            .unwrap_or(false)
    }

    let dir_errors: Vec<String> = chapter_dirs
        .par_iter()
        .filter_map(|chapter| {
            if !has_perms(chapter) {
                Some(format!(
                    "Directory {:?} lacks write permissions.",
                    chapter.file_name().unwrap_or_default()
                ))
            } else {
                None
            }
        })
        .collect();
    findings.negative.extend(dir_errors);

    for page in pages {
        if !has_perms(page) {
            findings.negative.push(format!(
                "File {:?} lacks write permissions.",
                page.file_name().unwrap_or_default()
            ));
        }
    }

    findings
}

fn validate_naming_convention(chapter_dirs: &[PathBuf]) -> (ValidationFindings, BundleFlag) {
    trace!("Checking directory naming conventions");
    let mut findings = ValidationFindings::new();

    let has_numeric: bool = chapter_dirs.par_iter().all(|path| {
        path.file_name()
            .and_then(|n| n.to_str())
            .map(|name| name.chars().any(char::is_numeric))
            .unwrap_or(false)
    });

    // Check for volume-chapter pattern using extract_volume_chapter
    let has_vol_chapter: bool = chapter_dirs.par_iter().all(|path| {
        let (vol, _ch) = scanner::extract_volume_chapter(path);
        vol.is_some()
    });

    let flag = if has_numeric && has_vol_chapter {
        findings.positive.push(
            "Directories correctly named and numbered. Automatic bundling will use the fastest algorithm.".into()
        );
        BundleFlag::Name
    } else {
        if !has_numeric {
            findings.warning.push(
                "Some directories lack numeric identifiers. Consider renaming for faster bundling."
                    .into(),
            );
        }
        if !has_vol_chapter {
            findings.warning.push(
                "Directory naming convention not followed; use 'VOLUME-CHAPTER' (e.g., '002-032') for faster bundling.".into()
            );
        }
        findings.warning.push(
            "Automatic bundling will use fallback mechanisms, potentially slowing the process."
                .into(),
        );
        BundleFlag::Image
    };

    (findings, flag)
}

fn validate_file_naming(pages: &[&PathBuf]) -> ValidationFindings {
    trace!("Checking file naming consistency");
    let mut findings = ValidationFindings::new();

    let consistently_named = pages.par_iter().all(|path| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| {
                let numeric_part: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
                !numeric_part.is_empty()
            })
            .unwrap_or(false)
    });

    if consistently_named {
        findings.positive.push(
            "All image files contain numeric identifiers. This ensures correct page ordering."
                .into(),
        );
    }

    findings
}

fn validate_image_count(pages: &[&PathBuf]) -> ValidationFindings {
    trace!("Checking image count");
    let mut findings = ValidationFindings::new();

    let count = pages.len();
    if count > 0 && count <= 10000 {
        findings.positive.push(format!(
            "Total of {} images found. Amount is within reasonable processing limits.",
            count
        ));
    } else if count > 10000 {
        findings.warning.push(format!(
            "Large number of images detected ({}). Processing may take a long time.",
            count
        ));
    }

    findings
}

fn validate_format_uniformity(pages: &[&PathBuf]) -> ValidationFindings {
    trace!("Checking image format consistency");
    let mut findings = ValidationFindings::new();

    let formats: std::collections::HashSet<String> = pages
        .par_iter()
        .filter_map(|p| p.extension().and_then(|e| e.to_str()))
        .map(|e| e.to_lowercase())
        .collect();

    if formats.len() == 1 {
        if let Some(fmt) = formats.iter().next() {
            findings.positive.push(format!(
                "All images use the same format ({}). This ensures consistent quality and processing.",
                fmt
            ));
        }
    }

    findings
}

fn validate_skipped_files(skipped: &[PathBuf]) -> ValidationFindings {
    let mut findings = ValidationFindings::new();

    if skipped.is_empty() {
        return findings;
    }

    let examples: Vec<String> = skipped
        .iter()
        .take(5)
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .collect();

    findings.warning.push(format!(
        "Found {} non-image files that will be skipped: {}",
        skipped.len(),
        examples.join(", ")
    ));

    findings
}
