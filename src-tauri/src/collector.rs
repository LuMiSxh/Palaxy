use std::cmp::Ordering;
use std::ffi::OsStr;
use std::path::PathBuf;

use image::{DynamicImage, GenericImageView, Pixel};
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use tauri::async_runtime::{JoinHandle, spawn};
use tokio::fs::{read_dir, ReadDir};

use crate::prelude::*;

pub struct Collector {
    base_directory: PathBuf,
}

lazy_static! {
    static ref RE: Regex = Regex::new(r"\d+\.?\d*").unwrap();
}


impl Collector {
    pub fn new(base_directory: &str) -> Self {
        Self {
            base_directory: PathBuf::from(base_directory)
        }
    }

    pub async fn collect_chapters(
        &mut self,
        comparator: Option<&'static (dyn Fn(&PathBuf, &PathBuf) -> Ordering + Sync)>,
    ) -> Result<Vec<PathBuf>, Error> {
        let mut chapters = Self::collect(&self.base_directory).await?;

        if let Some(comparator) = comparator {
            chapters.par_sort_by(comparator);
        }

        Ok(chapters)
    }

    pub async fn collect_pages(
        &self,
        chapters: Vec<PathBuf>,
        comparator: Option<&'static (dyn Fn(&PathBuf, &PathBuf) -> Ordering + Sync)>,
    ) -> Result<Vec<Vec<PathBuf>>, Error> {
        let mut pages = Vec::with_capacity(chapters.len());

        let handles: Vec<JoinHandle<Result<(usize, Vec<PathBuf>), Error>>> = chapters
            .into_par_iter()
            .enumerate()
            .map(|(index, chapter_dir)| {
                spawn(async move {
                    let mut chapter_images = Self::collect(&chapter_dir).await?;


                    if let Some(comparator) = comparator {
                        chapter_images.par_sort_by(comparator);
                    }

                    Ok((index, chapter_images))
                })
            })
            .collect();

        for handle in handles {
            let (i, chapter_images) = handle.await??;
            pages.insert(i, chapter_images);
        }

        Ok(pages)
    }

    pub async fn determine_volume_start_chapters(
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
                            if Collector::is_grayscale(&cover_image, sensibility)
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

    pub fn calculate_volume_sizes(
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

    // Helper methods

    pub fn is_grayscale(img: &DynamicImage, sensibility: f64) -> bool {
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

    pub fn check_path<F>(paths: &Vec<PathBuf>, test_case: F) -> Result<Vec<PathBuf>, Error>
        where F: Fn(&PathBuf) -> bool
    {
        let mut invalid_paths = Vec::new();

        for path in paths {
            if !test_case(path) {
                invalid_paths.push(path.clone());
            }
        }

        Ok(invalid_paths)
    }

    pub async fn collect(directory: &PathBuf) -> Result<Vec<PathBuf>, Error> {
        let mut entries: Vec<PathBuf> = Vec::new();
        let mut paths: ReadDir = read_dir(directory).await?;

        while let Some(path) = paths.next_entry().await? {
            // exclude hidden files
            if path.file_name().to_str().unwrap().starts_with(".") {
                continue;
            }

            entries.push(path.path());
        }

        Ok(entries)
    }

    pub fn sort_stem_by_name(a: &PathBuf, b: &PathBuf) -> Ordering {
        // This is a closure that takes a PathBuf and returns an Option<usize>.
        // Using this closure avoids copying the same code twice.
        let closure = |path: &PathBuf| -> Option<usize> {
            path.file_stem()?.to_str()?.parse::<usize>().ok()
        };

        closure(a).cmp(&closure(b))
    }

    fn regex_parser(s: &PathBuf) -> Option<f64> {
        let (capture, []) = RE
            .captures_iter(
                s.file_name()
                    .unwrap_or(
                        OsStr::new("")
                    )
                    .to_str()
                    .unwrap_or("")
            )
            .last()?
            .extract();

        Some(
            capture
                .trim_start_matches("0")
                .trim()
                .parse::<f64>()
                .ok()?
        )
    }

    pub fn sort_name_by_number(a: &PathBuf, b: &PathBuf) -> Ordering {
        let an = Self::regex_parser(a);
        let bn = Self::regex_parser(b);

        an.partial_cmp(&bn).unwrap()
    }
}