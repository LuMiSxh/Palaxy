use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use tauri::async_runtime::{spawn, spawn_blocking, JoinHandle};
use tokio::fs::create_dir;

use crate::collector::Collector;
use crate::generator::{cbz, epub, pdf};
use crate::prelude::*;

lazy_static! {
    static ref REGEX_ANALYZE: Regex = Regex::new(r"\d+-\d+(\.\d+)?").unwrap();
}

struct SharedData {
    name: String,
    target_directory: String,
    pages: Vec<Vec<PathBuf>>,
    chapters_per_volume: Vec<usize>,
}

// NEW METHOD
#[tauri::command(async)]
pub async fn flow_convert(
    source_directory: String,
    target_directory: String,
    chapters_per_volume: Vec<usize>,
    bundler_flag: BundlerFlag,
    create_directory: bool,
    file_format: FileFormat,
    direction: Direction,
) -> Result<FlowConvert, Error> {
    let now = std::time::Instant::now();

    // Specify the sorting method for the chapters
    let chapter_sorter: &'static (dyn Fn(&PathBuf, &PathBuf) -> Ordering + Sync) =
        match bundler_flag {
            BundlerFlag::NAME => &Collector::sort_by_name_volume_chapter,
            _ => &Collector::sort_name_by_number,
        };

    // Collect all chapters and pages
    let mut collector = Collector::new(&source_directory);
    let chapters: Vec<PathBuf> = collector.collect_chapters(Some(chapter_sorter)).await?;
    let pages: Vec<Vec<PathBuf>> = collector
        .collect_pages(chapters, Some(&Collector::sort_by_stem_number))
        .await?;

    // Specify the manga name
    let name: String = Path::new(&source_directory)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();

    // Create target folder if it doesn't exist but is requested,
    // else check if target directory exists
    let target_directory_path = match create_directory {
        true => {
            let path = Path::new(&target_directory).join(&name);
            if !path.exists() {
                create_dir(&path).await?;
            }
            Ok(path)
        }
        false => {
            let path = Path::new(&target_directory);
            if !path.exists() {
                Err(Error::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Target directory does not exist",
                )))
            } else {
                Ok(path.join(&name))
            }
        }
    }?
    .to_str()
    .unwrap()
    .to_string();

    let cpv = chapters_per_volume.clone();

    let data = Arc::new(SharedData {
        name,
        target_directory: target_directory_path,
        pages,
        chapters_per_volume,
    });

    let handles: Vec<JoinHandle<Result<(), Error>>> = cpv
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
            Err(e) => return Err(Error::from(e)),
        }
    }

    let elapsed = now.elapsed();
    Ok(FlowConvert {
        message: Some(format!("Finished in: {:.2?}", elapsed)),
    })
}

#[tauri::command(async)]
pub async fn flow_analyze(base_path: String) -> Result<FlowAnalyze, Error> {
    // Closures for use in this function
    let has_perms = |path: &PathBuf| {
        path.metadata()
            .map(|meta| !meta.permissions().readonly())
            .unwrap_or(false)
    };

    let is_valid = |path: &PathBuf| {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| match ext {
                "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" => true,
                _ => false,
            })
            .unwrap_or(false)
    };

    let mut negative: Vec<String> = Vec::new();
    let mut positive: Vec<String> = Vec::new();
    let mut suggest: Vec<String> = Vec::new();
    let mut bundler = BundlerFlag::IMAGE;
    let mut collector = Collector::new(&base_path);

    let chapters = collector.collect_chapters(None).await?;

    let mut pages = match chapters.is_empty() {
        true => Vec::new(),
        false => collector
            .collect_pages(chapters.clone(), None)
            .await?
            .concat(),
    };
    // Filter out all non-file paths
    pages = pages.into_iter().filter(|path| path.is_file()).collect();

    // Check if there are no subdirectories present
    if chapters.is_empty() {
        negative.push(String::from(
            "No subdirectories found in the base path. Ensure they exist with images inside.",
        ));
    }

    // Check if there are files
    if pages.is_empty() && !chapters.is_empty() {
        negative.push(String::from(
            "Subdirectories contain no files. Verify that chapter images are placed inside.",
        ));
    }

    // Check if the directories contain numbers
    let inv_dir_num = Collector::check_path(&chapters, |path| {
        path.file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .chars()
            .any(char::is_numeric)
    })?;
    // If there are directories without numbers,
    // add one to the negative list as an example with an explanation as to why
    if !inv_dir_num.is_empty() {
        negative.push(
            format!("Directory '{:?}' lacks numerical identifiers. \nRequired for sorting algorithm functionality.",
                    inv_dir_num[0].file_name().unwrap()
            )
        );
    }

    // Check if the chapters have the required permissions
    for chapter in &chapters {
        if !has_perms(chapter) {
            negative.push(format!(
                "Directory '{:?}' lacks write permissions. \nRequired for full functionality.",
                chapter.file_name().unwrap()
            ));
            // Break the loop if one directory is invalid. No need to check the rest.
            break;
        }
    }

    // Check if the images have a valid extension
    for page in &pages {
        if !is_valid(page) {
            negative.push(
                format!("File '{:?}' has an invalid extension. \nSupported formats: jpg, jpeg, png, gif, webp, bmp.",
                        page.file_name().unwrap()
                )
            );
            // Break the loop if one file is invalid. No need to check the rest.
            break;
        }
    }

    // Check if the directories are using the correct naming convention
    let inv_dir_naming = Collector::check_path(&chapters, |path| {
        REGEX_ANALYZE.is_match(path.file_name().unwrap().to_str().unwrap())
    })?;
    // If there are directories without the correct naming convention,
    // add an example with the naming convention to the suggest list
    if !inv_dir_naming.is_empty() {
        suggest.push(
            String::from(
                "Subdirectory naming convention not followed; use 'VOLUME-CHAPTER' (e.g., '002-032') for faster bundling."
            )
        );
    }
    // If all directories are correct,
    // add a positive message mentioning that the automatic bundling will work
    if inv_dir_num.is_empty() && inv_dir_naming.is_empty() {
        positive.push(
            String::from(
                "Directories correctly named and numbered. Automatic bundling will proceed without fallback mechanisms."
            )
        );
        bundler = BundlerFlag::NAME;
    } else {
        positive.push(
            String::from(
                "Automatic bundling will use fallback mechanisms, potentially slowing the process and increasing error risk."
            )
        );
    }

    // Check if the files are only using numbers as their name
    let inv_file_num = Collector::check_path(&pages, |path| {
        path.file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .replace(".", "")
            .chars()
            .all(char::is_numeric)
    })?;
    // If there are files without numbers,
    // add one to the negative list as an example with an explanation as to why
    if !inv_file_num.is_empty() {
        negative.push(format!(
            "File '{:?}' lacks numerical naming. Required for effective sorting and bundling.",
            inv_file_num[0].file_name().unwrap()
        ));
    }

    Ok(FlowAnalyze {
        message: None,
        negative,
        positive,
        suggest,
        bundler,
    })
}

#[tauri::command(async)]
pub async fn flow_volume(
    base_path: String,
    bundler_flag: BundlerFlag,
    sensibility: Option<usize>,
) -> Result<FlowVolume, Error> {
    let now = std::time::Instant::now();

    let mut collector = Collector::new(&base_path);

    let mut chapters: Vec<PathBuf> = collector
        .collect_chapters(if bundler_flag == BundlerFlag::IMAGE {
            Some(&Collector::sort_name_by_number)
        } else {
            None
        })
        .await?;

    let pages: Vec<Vec<PathBuf>> = collector
        .collect_pages(chapters.clone(), Some(&Collector::sort_by_stem_number))
        .await?;

    let total_chapters: usize = chapters.len();
    let mut total_volumes: Option<usize> = None;
    let mut chapters_per_volume: Option<Vec<usize>> = None;

    match bundler_flag {
        // For manual bundling, the user will have to manually input the remaining information.
        // This will be done in the frontend.
        BundlerFlag::MANUAL => {}
        // For automatic bundling by name,
        // the program will use the naming convention
        // to determine the volumes and chapters.
        BundlerFlag::NAME => {
            let mut tmp: Vec<usize> = Vec::new();
            let mut extra: bool = false;

            // Step 1: Sort the chapters by their chapter number
            chapters.par_sort_by(Collector::sort_by_name_volume_chapter);

            // Step 2: Determine the start of each volume
            for (i, chapter) in chapters.iter().enumerate() {
                let closure = |path: &PathBuf| -> Option<usize> {
                    path.file_name()?
                        .to_str()?
                        .split("-")
                        .next()?
                        .parse::<usize>()
                        .ok()
                };

                let volume_number = closure(chapter).unwrap();

                // If the volume number is 0, it is not counted as start volume. It needs to come last
                if volume_number == 0 {
                    extra = true;
                }

                // If the volume number is smaller than the total of volume start chapters, add it
                if tmp.len() < volume_number {
                    tmp.push(i);
                }
            }

            // If there is an extra chapter, add it to the end
            if extra {
                tmp.push(0);
            }

            // Step 3:
            // Calculate the number chapters per volume based on the start chapters of each volume
            let mut tmp2: Vec<usize> = Vec::new();
            for i in 0..tmp.len() {
                if i == tmp.len() - 1 {
                    tmp2.push(chapters.len() - tmp[i]);
                } else {
                    tmp2.push(tmp[i + 1] - tmp[i]);
                }
            }

            // Step 4: Set the correct variables
            chapters_per_volume = Some(tmp2);
            total_volumes = Some(tmp.len());
        }
        BundlerFlag::IMAGE => {
            let volume_start_chapters: Vec<usize> = collector
                .determine_volume_start_chapters(
                    pages,
                    if let Some(sensibility) = sensibility {
                        sensibility as f64 / 100.0
                    } else {
                        0.75f64
                    },
                )
                .await?;

            total_volumes = Some(volume_start_chapters.len());

            let volume_sizes =
                collector.calculate_volume_sizes(volume_start_chapters, total_chapters)?;

            chapters_per_volume = Some(volume_sizes);
        }
    };

    let elapsed = now.elapsed();

    Ok(FlowVolume {
        message: Some(format!("Finished in: {:.2?}", elapsed)),
        total_chapters,
        total_volumes,
        chapters_per_volume,
    })
}
