use std::path::PathBuf;

use image::{DynamicImage, GenericImageView, Pixel};
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use tauri::async_runtime::{JoinHandle, spawn};
use tokio::fs::{read_dir, ReadDir};

use crate::prelude::*;

lazy_static! {
    static ref RE: Regex = Regex::new(r"\d+\.?\d*").unwrap();
}

fn regex_parser(s: &PathBuf) -> Result<&str, Error> {
    let (capture, []) = RE.captures_iter(s.file_name().unwrap().to_str().unwrap())
        .last()
        .ok_or(
            Error::Unsupported("Failed to retrieve last capture group from regex match".to_string()))
        ?
        .extract();

    Ok(capture.trim_start_matches("0").trim())
}

fn regex_sorter(a: &PathBuf, b: &PathBuf) -> std::cmp::Ordering {
    let an = regex_parser(a).expect("Failed to parse PathBuf as &str");
    let bn = regex_parser(b).expect("Failed to parse PathBuf as &str");

    an.partial_cmp(bn).unwrap()
}

async fn collect_entries(dir: &PathBuf) -> Result<Vec<PathBuf>, Error> {
    let mut entries: Vec<PathBuf> = Vec::new();
    let mut paths: ReadDir = read_dir(dir).await?;

    while let Some(path) = paths.next_entry().await? {
        if let Some(file_name) = path.file_name().to_str() {
            if file_name.starts_with('.') || (path.path().is_dir() && !RE.is_match(&file_name)) {
                continue;
            }
        } else {
            continue;
        }

        entries.push(path.path());
    }

    Ok(entries)
}

pub fn is_mostly_grayscale(img: &DynamicImage, sensibility: f64) -> bool {
    let total_pixels = (img.width() * img.height()) as f64;
    let gray_threshold = total_pixels * sensibility;

    let gray_pixels: usize = img
        .pixels()
        .par_bridge()
        .filter(|&(_, _, pixel)| {
            let rgb = pixel.to_rgb();
            let r = rgb[0];
            let g = rgb[1];
            let b = rgb[2];

            // A simple heuristic: if all color channels are approximately equal, the pixel is likely grayscale.
            (r.wrapping_sub(g) < 10) && (r.wrapping_sub(b) < 10) && (g.wrapping_sub(b) < 10)
        })
        .count();

    gray_pixels > gray_threshold as usize
}

pub struct Collector {
    root_directory: PathBuf,
}

impl Collector {
    pub fn new(root_directory: &str) -> Self {
        Self {
            root_directory: PathBuf::from(root_directory)
        }
    }

    pub async fn collect_chapters(&mut self) -> Result<Vec<PathBuf>, Error> {
        let mut chapters = collect_entries(&self.root_directory).await?;
        chapters.par_sort_by(regex_sorter);
        Ok(chapters)
    }

    pub async fn collect_pages(&self, chapters: Vec<PathBuf>) -> Result<Vec<Vec<PathBuf>>, Error> {
        let mut images = Vec::with_capacity(chapters.len());

        let handles: Vec<JoinHandle<Result<(usize, Vec<PathBuf>), Error>>> = chapters
            .into_par_iter()
            .enumerate()
            .map(|(index, chapter_dir)| {
                spawn(async move {
                    let mut chapter_images = collect_entries(&chapter_dir).await?;
                    chapter_images.par_sort_by(regex_sorter);
                    Ok((index, chapter_images))
                })
            })
            .collect();

        for handle in handles {
            let (i, chapter_images) = handle.await??;
            images.insert(i, chapter_images);
        }

        Ok(images)
    }

    pub async fn collect_volume_start_chapters(
        &self,
        images_per_chapter: Vec<Vec<PathBuf>>,
        sensibility: f64,
    ) -> Result<Vec<usize>, Error> {
        let mut book_start_chapters: Vec<usize> = Vec::new();

        let handles: Vec<JoinHandle<Result<Option<usize>, Error>>> =
            images_per_chapter
                .into_par_iter()
                .enumerate()
                .map(|(i, images_per_chapter)| {
                    spawn(async move {
                        let cover_path = &images_per_chapter[0];
                        let cover_image = image::open(cover_path)?;

                        Ok(
                            if is_mostly_grayscale(&cover_image, sensibility)
                            { None } else { Some(i) }
                        )
                    })
                }).collect();

        for handle in handles {
            if let Ok(result) = handle.await? {
                if let Some(i) = result {
                    book_start_chapters.push(i);
                }
            }
        }

        // Sort the chapters by their starting index.
        book_start_chapters.sort();

        Ok(book_start_chapters)
    }

    pub fn calculate_book_chapters(
        &self,
        mut book_start_chapters: Vec<usize>,
        total_chapters: usize,
    ) -> Result<Vec<usize>, Error> {
        let mut book_chapters: Vec<usize> = Vec::new();

        // Remove the first chapter because it's always a book start
        if book_start_chapters.len() > 0 {
            book_start_chapters.remove(0);
        } else {
            return Err(Error::NotFound("No chapters found".to_string()));
        }

        let mut prev_chapter = 0;
        for chapter in book_start_chapters {
            book_chapters.push(chapter - prev_chapter);
            prev_chapter = chapter;
        }

        // Add the remaining chapters.
        book_chapters.push(total_chapters - prev_chapter);

        Ok(book_chapters)
    }
}
