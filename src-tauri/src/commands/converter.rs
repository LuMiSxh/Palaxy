use crate::collector::Collector;
use crate::generator::{cbz, epub, pdf};
use crate::prelude::*;
use crate::types::{
    AnalyzeResponse, BaseResponse, BundleFlag, BundleResponse, CommAnalyzeMeta, CommBundle,
    ConvStateKey, Direction, FileFormat,
};
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::async_runtime::{spawn, spawn_blocking, JoinHandle};
use tauri::State;
use tokio::fs::create_dir;
use tokio::sync::Mutex;

lazy_static! {
    static ref REGEX_ANALYZE: Regex = Regex::new(r"\d+-\d+(\.\d+)?").unwrap();
}

// ConvState

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

    dir_lacks_numeric.iter().for_each(|dir| {
        negative.push(format!(
            "Directory {:?} lacks numerical identifiers. Remove them for faster bundling.",
            dir.file_name().unwrap()
        ));
    });

    // Image Format Validation
    let unsupported_formats = Collector::check_path(&pages, |path| {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some(ext) => matches!(ext.to_lowercase().as_str(), "jpg" | "jpeg" | "png" | "webp"),
            None => false, // Files without extensions are unsupported
        }
    })?;

    if !unsupported_formats.is_empty() {
        let format_examples: Vec<String> = unsupported_formats
            .iter()
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
        .iter()
        .filter_map(|p| p.metadata().ok().map(|m| m.len()))
        .collect();

    if !file_sizes.is_empty() {
        let avg_size = file_sizes.iter().sum::<u64>() / file_sizes.len() as u64;
        let outliers: Vec<_> = file_sizes
            .iter()
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
            let chapter_file_counts: Vec<usize> = pages_vec.iter().map(|p| p.len()).collect();

            if !chapter_file_counts.is_empty() {
                let avg_count =
                    chapter_file_counts.iter().sum::<usize>() / chapter_file_counts.len();
                let outliers = chapter_file_counts
                    .iter()
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
        .iter()
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
        .iter()
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

    chapters.iter().for_each(|chapter| {
        if !has_perms(chapter) {
            negative.push(format!(
                "Directory {:?} lacks write permissions. Required for full functionality.",
                chapter.file_name().unwrap()
            ));
        }
    });

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
        .iter()
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
        .iter()
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

#[tauri::command(async)]
#[specta::specta]
pub async fn conv_bundle(
    sensibility: Option<usize>,
    state: State<'_, Mutex<ConvState>>,
) -> EResult<CommBundle> {
    let start = std::time::Instant::now();

    let mut state = state.lock().await;
    let mut collector = Collector::new(&state.source);

    // Collect all pages and sort based on bundle_flag
    let mut chapters: Vec<PathBuf> = collector
        .collect_chapters(if state.bundle_flag == BundleFlag::Image {
            Some(&Collector::sort_name_by_number)
        } else {
            None
        })
        .await?;

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

            // Sort the chapters by their chapter number
            chapters.par_sort_by(Collector::sort_by_name_volume_chapter);

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

struct SharedData {
    name: String,
    target_directory: String,
    pages: Vec<Vec<PathBuf>>,
    chapters_per_volume: Vec<usize>,
}

#[tauri::command(async)]
#[specta::specta]
pub async fn conv_convert(state: State<'_, Mutex<ConvState>>) -> EResult<BaseResponse> {
    let start = std::time::Instant::now();

    // Extract all needed data while the lock is held
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

    let shared_data = Arc::new(SharedData {
        name,
        target_directory: target_directory_path,
        pages,
        chapters_per_volume: volume_sizes.clone(),
    });

    let handles: Vec<JoinHandle<Result<(), Error>>> = volume_sizes
        .clone()
        .into_iter()
        .enumerate()
        .map(|(i, chapters)| {
            let data = Arc::clone(&shared_data);

            // Spawn a new thread for each volume but make sure to use the correct spawning method
            match format {
                FileFormat::Cbz => spawn_blocking(move || {
                    let j: usize = data.chapters_per_volume[0..i].par_iter().sum();

                    let volume_name = format!("{} | {}", data.name, i + 1);

                    let mut cbz = cbz::Cbz::new(&data.target_directory, &volume_name)?;

                    for k in j..(j + chapters) {
                        for page in &data.pages[k] {
                            cbz.add_page(page)?;
                        }
                    }

                    cbz.set_comicinfo(&volume_name, i + 1)?;
                    cbz.save()?;

                    Ok(())
                }),
                FileFormat::Epub => spawn(async move {
                    let j: usize = data.chapters_per_volume[0..i].par_iter().sum();

                    let volume_name = format!("{} | {}", data.name, i + 1);

                    let mut epub = epub::EPub::new()?;

                    epub.set_cover(&data.pages[j][0])?
                        .set_lang("en")?
                        .set_metadata("title", &volume_name)?
                        .set_metadata("author", "Manga Bundler")?
                        .set_metadata(
                            "direction",
                            if direction == Direction::Ltr {
                                "ltr"
                            } else {
                                "rtl"
                            },
                        )?;

                    for k in j..(j + chapters) {
                        epub.add_chapter(k + 1, &data.pages[k]).await?;
                    }

                    epub.save(&data.target_directory, format!("{}", volume_name).as_str())
                        .await?;

                    Ok(())
                }),
                FileFormat::Pdf => spawn_blocking(move || {
                    let j: usize = data.chapters_per_volume[0..i].par_iter().sum();

                    let volume_name = format!("{} | {}", data.name, i + 1);

                    let mut pdf = pdf::Pdf::new(&volume_name, &data.pages[j][0])?;

                    for k in (j + 1)..(j + chapters) {
                        for page in &data.pages[k] {
                            pdf.add_page(page)?;
                        }
                    }

                    pdf.save(&data.target_directory, &volume_name)?;

                    Ok(())
                }),
            }
        })
        .collect();

    // Wait for all threads to finish
    for handle in handles {
        match handle.await {
            Ok(_) => {}
            Err(e) => return Err(Error::AsyncTaskError(e.to_string())),
        }
    }

    Ok(BaseResponse::default_duration(
        start.elapsed().as_secs_f64(),
    ))
}
