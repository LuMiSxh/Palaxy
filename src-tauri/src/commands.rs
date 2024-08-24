use crate::collector::Collector;
use crate::generator::{cbz, epub, pdf};
use crate::prelude::*;
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

// -- RESET --

#[tauri::command(async)]
pub async fn reset(state: State<'_, Mutex<AppState>>) -> EResult<CommandDefault> {
    let mut state = state.lock().await;
    state.reset();
    Ok(CommandDefault::default())
}

// -- SETTER --

#[tauri::command(async)]
pub async fn set_source(source: String, state: State<'_, Mutex<AppState>>) -> EResult<CommandDefault> {
    let mut state = state.lock().await;
    state.source = PathBuf::from(source);

    state.name = state.source
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_string())
        .unwrap_or_else(|| "Palaxy-Converted".to_string());

    Ok(CommandDefault::default())
}

#[tauri::command(async)]
pub async fn set_volume_sizes(sizes: Vec<usize>, state: State<'_, Mutex<AppState>>) -> EResult<CommandDefault> {
    let mut state = state.lock().await;
    state.volume_sizes = sizes;
    Ok(CommandDefault::default())
}

#[tauri::command(async)]
pub async fn set_bundle_flag(flag: BundleFlag, state: State<'_, Mutex<AppState>>) -> EResult<CommandDefault> {
    let mut state = state.lock().await;
    state.bundle_flag = flag;
    Ok(CommandDefault::default())
}

#[tauri::command(async)]
pub async fn set_data(data: Vec<Vec<PathBuf>>, state: State<'_, Mutex<AppState>>) -> EResult<CommandDefault> {
    let mut state = state.lock().await;
    state.data = data;
    Ok(CommandDefault::default())
}

// -- GETTER --

#[tauri::command(async)]
pub async fn get_data(state: State<'_, Mutex<AppState>>) -> EResult<CommandGetData> {
    let state = state.lock().await;
    Ok(CommandGetData {
        message: None,
        data: state.data.clone(),
    })
}

// -- PROCESSES --

#[tauri::command(async)]
pub async fn analyze(state: State<'_, Mutex<AppState>>) -> EResult<CommandAnalyze> {
    let state = state.lock().await;

    fn has_perms(path: &PathBuf) -> bool {
        path.metadata()
            .map(|meta| !meta.permissions().readonly())
            .unwrap_or(false)
    }

    let mut negative = Vec::new();
    let mut positive = Vec::new();
    let mut suggest = Vec::new();
    let mut flag = BundleFlag::IMAGE;
    let mut collector = Collector::new(&state.source);

    let chapters = collector.collect_chapters(None).await?;
    let mut pages = match chapters.is_empty() {
        true => Vec::new(),
        false => collector
            .collect_pages(chapters.clone(), None)
            .await?
            .concat(),
    };

    pages.retain(|path| path.is_file());

    if chapters.is_empty() {
        negative.push(
            "No subdirectories found in the base path. Ensure they exist with images inside."
                .to_string(),
        );

        return Ok(CommandAnalyze {
            message: None,
            negative,
            positive,
            suggest,
            flag,
        });
    }

    if pages.is_empty() && !chapters.is_empty() {
        negative.push(
            "Subdirectories contain no files. Verify that chapter images are placed inside."
                .to_string(),
        );

        return Ok(CommandAnalyze {
            message: None,
            negative,
            positive,
            suggest,
            flag,
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
        suggest.push("Subdirectory naming convention not followed; use 'VOLUME-CHAPTER' (e.g., '002-032') for faster bundling.".to_string());
    }

    if dir_lacks_numeric.is_empty() && dir_lacks_naming.is_empty() {
        positive.push("Directories correctly named and numbered. Automatic bundling will proceed with the fastest algorithm.".to_string());
        flag = BundleFlag::NAME;
    } else {
        positive.push("Automatic bundling will use fallback mechanisms, potentially slowing the process and increasing error risk.".to_string());
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

    Ok(CommandAnalyze {
        message: None,
        negative,
        positive,
        suggest,
        flag,
    })
}

#[tauri::command(async)]
pub async fn bundle(
    sensibility: Option<usize>,
    state: State<'_, Mutex<AppState>>,
) -> EResult<CommandBundle> {
    let now = std::time::Instant::now();
    let mut state = state.lock().await;
    let mut collector = Collector::new(&state.source);

    // Collect all pages and sort based on bundle_flag
    let mut chapters: Vec<PathBuf> = collector
        .collect_chapters(if state.bundle_flag == BundleFlag::IMAGE {
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
        BundleFlag::MANUAL => {}
        // For automatic bundling by name,
        // the program will use the naming convention
        // to determine the volumes and chapters.
        BundleFlag::NAME => {
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
        BundleFlag::IMAGE => {
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

    Ok(CommandBundle {
        message: Some(format!(
            "Bundling completed in {:.2?} seconds.",
            now.elapsed().as_secs_f64()
        )),
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
    })
}

struct SharedData {
    name: String,
    target_directory: String,
    pages: Vec<Vec<PathBuf>>,
    chapters_per_volume: Vec<usize>,
}

#[tauri::command(async)]
pub async fn convert(
    create_directory: bool,
    target: String,
    file_format: FileFormat,
    direction: Direction,
    state: State<'_, Mutex<AppState>>,
) -> EResult<CommandDefault> {
    let now = std::time::Instant::now();
    let state = state.lock().await;

    // Get all the state data needed
    let target_directory_path = match create_directory {
        true => {
            let path = Path::new(&target).join(&state.name);
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

    let data = Arc::new(SharedData {
        name: state.name.clone(),
        target_directory: target_directory_path,
        pages: state.data.clone(),
        chapters_per_volume: state.volume_sizes.clone(),
    });

    let handles: Vec<JoinHandle<Result<(), Error>>> = state
        .volume_sizes
        .clone()
        .into_iter()
        .enumerate()
        .map(|(i, chapters)| {
            let data = Arc::clone(&data);

            // Spawn a new thread for each volume but make sure to use the correct spawning method
            match file_format {
                FileFormat::CBZ => spawn_blocking(move || {
                    let j: usize = data.chapters_per_volume[0..i].par_iter().sum();

                    let volume_name = format!("{} | {}", data.name, i + 1);

                    let mut cbz = cbz::Cbz::new(&data.target_directory, &volume_name)?;

                    for k in j..(j + chapters) {
                        for page in &data.pages[k] {
                            cbz.add_page(page)?;
                        }
                    }

                    cbz.set_comicinfo(&volume_name, i + 1)?.save()?;

                    Ok(())
                }),
                FileFormat::EPUB => spawn(async move {
                    let j: usize = data.chapters_per_volume[0..i].par_iter().sum();

                    let volume_name = format!("{} | {}", data.name, i + 1);

                    let mut epub = epub::EPub::new()?;

                    epub.set_cover(&data.pages[j][0])?
                        .set_lang("en")?
                        .set_metadata("title", &volume_name)?
                        .set_metadata("author", "Manga Bundler")?
                        .set_metadata(
                            "direction",
                            if direction == Direction::LTR {
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
                FileFormat::PDF => spawn_blocking(move || {
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

    let elapsed = now.elapsed();

    Ok(CommandDefault {
        message: Some(format!(
            "Conversion completed in {:.2?} seconds.",
            elapsed.as_secs_f64()
        )),
    })
}
