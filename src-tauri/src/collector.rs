//! Comic/manga image collection and organization module.
//!
//! This module provides functionality to collect, organize and analyze image files
//! from a directory structure, typically representing chapters and pages of comics or manga.
//! It includes tools for sorting files numerically and detecting chapter boundaries.

use std::cmp::Ordering;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::Arc;

use image::{DynamicImage, GenericImageView, Pixel};
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use tauri::async_runtime::{spawn, spawn_blocking, JoinHandle};
use tokio::fs::{read_dir, ReadDir};
use tokio::sync::Semaphore;

use crate::prelude::*;

/// Limits the number of concurrent directory operations
const MAX_CONCURRENT_DIRS: usize = 64;
/// Controls how many pixels to skip when sampling for grayscale detection
const GRAYSCALE_SAMPLE_RATE: u32 = 10;
/// Maximum dimension for grayscale detection before downsampling
const GRAYSCALE_MAX_DIMENSION: u32 = 500;

/// Manages collection and organization of image files in a directory structure
pub struct Collector {
    /// Root directory containing chapters or volumes
    base_directory: PathBuf,
}

lazy_static! {
    /// Regex pattern for extracting numeric values from filenames
    static ref RE: Regex = Regex::new(r"\d+\.?\d*").unwrap();
}

impl Collector {
    /// Creates a new Collector instance for the specified directory
    ///
    /// # Arguments
    ///
    /// * `base_directory` - Path to the root directory containing chapters/volumes
    pub fn new(base_directory: &PathBuf) -> Self {
        Self {
            base_directory: base_directory.clone(),
        }
    }

    /// Collects chapter directories from the base directory
    ///
    /// # Arguments
    ///
    /// * `comparator` - Optional function to sort the collected chapters
    ///
    /// # Returns
    ///
    /// * `EResult<Vec<PathBuf>>` - Vector of paths to chapter directories
    pub async fn collect_chapters(
        &mut self,
        comparator: Option<&'static (dyn Fn(&PathBuf, &PathBuf) -> Ordering + Sync)>,
    ) -> EResult<Vec<PathBuf>> {
        let mut chapters = Self::collect_parallel(&self.base_directory, true).await?;

        if let Some(comparator) = comparator {
            chapters.par_sort_by(comparator);
        }

        Ok(chapters)
    }

    /// Collects page images from each chapter directory
    ///
    /// # Arguments
    ///
    /// * `chapters` - Vector of chapter directory paths
    /// * `comparator` - Optional function to sort the collected pages
    ///
    /// # Returns
    ///
    /// * `EResult<Vec<Vec<PathBuf>>>` - Vector of vectors containing page paths for each chapter
    pub async fn collect_pages(
        &self,
        chapters: Vec<PathBuf>,
        comparator: Option<&'static (dyn Fn(&PathBuf, &PathBuf) -> Ordering + Sync)>,
    ) -> EResult<Vec<Vec<PathBuf>>> {
        // Create semaphore to limit concurrent tasks
        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_DIRS));

        // Create a vector to hold EResults, pre-allocate with capacity
        let mut pages = Vec::with_capacity(chapters.len());
        for _ in 0..chapters.len() {
            pages.push(Vec::new());
        }

        let handles: Vec<JoinHandle<EResult<(usize, Vec<PathBuf>)>>> = chapters
            .into_par_iter()
            .enumerate()
            .map(|(index, chapter_dir)| {
                let semaphore = Arc::clone(&semaphore);
                let chapter_dir = chapter_dir.clone();

                spawn(async move {
                    // Acquire semaphore permit to limit concurrent directory operations
                    let _permit = semaphore.acquire().await.map_err(|e| {
                        Error::AsyncTaskError(format!("Failed to acquire semaphore: {}", e))
                    })?;

                    let mut chapter_images = Self::collect_parallel(&chapter_dir, false).await?;

                    if let Some(comparator) = comparator {
                        chapter_images.par_sort_by(comparator);
                    }

                    Ok((index, chapter_images))
                })
            })
            .collect();

        for handle in handles {
            match handle.await {
                Ok(Ok((i, chapter_images))) => pages[i] = chapter_images,
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(Error::AsyncTaskError(e.to_string())),
            }
        }

        Ok(pages)
    }

    /// Identifies chapters that are likely to be the start of a new volume
    /// by analyzing the cover image (first image) of each chapter
    ///
    /// # Arguments
    ///
    /// * `images_per_chapter` - Nested vector of image paths organized by chapter
    /// * `sensibility` - Threshold value for grayscale detection sensitivity
    ///
    /// # Returns
    ///
    /// * `EResult<Vec<usize>>` - Indices of chapters that start new volumes
    pub async fn determine_volume_start_chapters(
        &self,
        images_per_chapter: Vec<Vec<PathBuf>>,
        sensibility: f64,
    ) -> EResult<Vec<usize>> {
        let mut book_start_chapters: Vec<usize> = Vec::new();

        // Limit concurrent image processing tasks
        let semaphore = Arc::new(Semaphore::new(num_cpus::get()));

        let handles: Vec<JoinHandle<EResult<Option<usize>>>> = images_per_chapter
            .into_par_iter()
            .enumerate()
            .map(|(i, images_per_chapter)| {
                if images_per_chapter.is_empty() {
                    return spawn_blocking(move || Ok(None));
                }

                let cover_path = images_per_chapter[0].clone();
                let semaphore = Arc::clone(&semaphore);

                spawn(async move {
                    // Acquire permit to limit concurrent image processing
                    let _permit = semaphore.acquire().await.map_err(|e| {
                        Error::AsyncTaskError(format!("Failed to acquire semaphore: {}", e))
                    })?;

                    let result = spawn_blocking(move || {
                        let cover_image = image::open(&cover_path)?;
                        Ok(if Collector::is_grayscale(&cover_image, sensibility) {
                            None
                        } else {
                            Some(i)
                        })
                    })
                    .await
                    .map_err(|e| Error::AsyncTaskError(e.to_string()))?;

                    result
                })
            })
            .collect();

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

    /// Calculates how many chapters belong to each volume
    ///
    /// # Arguments
    ///
    /// * `book_start_chapters` - Vector of chapter indices that start new volumes
    /// * `total_chapters` - Total number of chapters
    ///
    /// # Returns
    ///
    /// * `EResult<Vec<usize>>` - Vector of chapter counts for each volume
    pub fn calculate_volume_sizes(
        &self,
        mut book_start_chapters: Vec<usize>,
        total_chapters: usize,
    ) -> EResult<Vec<usize>> {
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

    /// Determines whether an image is predominantly grayscale
    ///
    /// # Arguments
    ///
    /// * `img` - Dynamic image to analyze
    /// * `sensibility` - Threshold value (0.0-1.0) determining how many pixels must be gray
    ///
    /// # Returns
    ///
    /// * `bool` - True if the image is predominantly grayscale
    pub fn is_grayscale(img: &DynamicImage, sensibility: f64) -> bool {
        // Downsample image if it's too large to improve performance
        let img = if img.width() > GRAYSCALE_MAX_DIMENSION || img.height() > GRAYSCALE_MAX_DIMENSION
        {
            let scale = GRAYSCALE_MAX_DIMENSION as f32 / img.width().max(img.height()) as f32;
            let new_width = (img.width() as f32 * scale) as u32;
            let new_height = (img.height() as f32 * scale) as u32;
            img.thumbnail(new_width, new_height)
        } else {
            img.clone()
        };

        let total_pixels = (img.width() * img.height()) as f64;
        let gray_threshold = total_pixels * sensibility;

        // Create chunks of pixels to process in parallel
        let width = img.width();
        let height = img.height();

        // Consider only every Nth pixel to speed up processing
        let samples = (0..height)
            .step_by(GRAYSCALE_SAMPLE_RATE as usize)
            .flat_map(|y| {
                (0..width)
                    .step_by(GRAYSCALE_SAMPLE_RATE as usize)
                    .map(move |x| (x, y))
            })
            .collect::<Vec<_>>();

        let sample_count = samples.len();

        let gray_pixels = samples
            .par_iter()
            .map(|(x, y)| {
                let pixel = img.get_pixel(*x, *y);
                let rgb = pixel.to_rgb();
                let r = rgb[0];
                let g = rgb[1];
                let b = rgb[2];

                // Using a more efficient calculation for grayscale detection
                // Check if the RGB values are close to each other
                let r_diff = r.abs_diff(g);
                let g_diff = g.abs_diff(b);
                let b_diff = b.abs_diff(r);

                r_diff <= 10 && g_diff <= 10 && b_diff <= 10
            })
            .filter(|&is_gray| is_gray)
            .count();

        // Scale back to estimate the full image
        let estimated_gray_pixels = (gray_pixels as f64 * total_pixels) / sample_count as f64;

        estimated_gray_pixels > gray_threshold
    }

    /// Collects directory contents in parallel with filtering options
    ///
    /// # Arguments
    ///
    /// * `directory` - Directory to scan
    /// * `only_dirs` - When true, only directories are collected; when false, only files
    ///
    /// # Returns
    ///
    /// * `EResult<Vec<PathBuf>>` - Paths meeting the criteria
    pub async fn collect_parallel(directory: &PathBuf, only_dirs: bool) -> EResult<Vec<PathBuf>> {
        let mut entries: Vec<PathBuf> = Vec::new();

        // Read directory contents
        let mut paths: ReadDir = read_dir(directory).await?;

        while let Some(entry) = paths.next_entry().await? {
            let path = entry.path();

            // Skip hidden files
            if let Some(file_name) = path.file_name() {
                if file_name.to_string_lossy().starts_with(".") {
                    continue;
                }
            }

            // Apply directory/file filter
            let is_dir = path.is_dir();
            if (only_dirs && !is_dir) || (!only_dirs && is_dir) {
                return Err(Error::InvalidPath(
                    path.clone(),
                    format!(
                        "{} expected, got {}",
                        if only_dirs { "Directory" } else { "File" },
                        if is_dir { "directory" } else { "file" }
                    ),
                ));
            }

            entries.push(path);
        }

        Ok(entries)
    }

    /// Filters paths based on a test condition
    ///
    /// # Arguments
    ///
    /// * `paths` - Vector of paths to check
    /// * `test_case` - Function that returns true if the path passes the test
    ///
    /// # Returns
    ///
    /// * `EResult<Vec<PathBuf>>` - Paths that failed the test
    pub fn check_path<F>(paths: &Vec<PathBuf>, test_case: F) -> EResult<Vec<PathBuf>>
    where
        F: Fn(&PathBuf) -> bool + Sync + Send,
    {
        let invalid_paths: Vec<PathBuf> = paths
            .par_iter()
            .filter(|path| !test_case(path))
            .cloned()
            .collect();

        Ok(invalid_paths)
    }

    /// Sorts paths by numeric values in their file stem
    ///
    /// # Arguments
    ///
    /// * `a` - First path to compare
    /// * `b` - Second path to compare
    ///
    /// # Returns
    ///
    /// * `Ordering` - Ordering based on numeric file stem values
    pub fn sort_by_stem_number(a: &PathBuf, b: &PathBuf) -> Ordering {
        // Cache the parsed numbers for better performance
        fn parse_number(path: &PathBuf) -> Option<usize> {
            path.file_stem()?.to_str()?.parse::<usize>().ok()
        }

        parse_number(a).cmp(&parse_number(b))
    }

    /// Extracts a numeric value from a path using regex
    ///
    /// # Arguments
    ///
    /// * `s` - Path to extract number from
    ///
    /// # Returns
    ///
    /// * `Option<f64>` - Extracted number or None if not found
    fn regex_parser(s: &PathBuf) -> Option<f64> {
        let file_name = s
            .file_name()
            .unwrap_or(OsStr::new(""))
            .to_str()
            .unwrap_or("");

        RE.captures_iter(file_name)
            .last()
            .map(|cap| {
                let capture = cap.get(0).unwrap().as_str();
                capture.trim_start_matches("0").trim().parse::<f64>().ok()
            })
            .flatten()
    }

    /// Sorts paths by numeric values found in their names
    ///
    /// # Arguments
    ///
    /// * `a` - First path to compare
    /// * `b` - Second path to compare
    ///
    /// # Returns
    ///
    /// * `Ordering` - Ordering based on numeric values in filenames
    pub fn sort_name_by_number(a: &PathBuf, b: &PathBuf) -> Ordering {
        let an = Self::regex_parser(a);
        let bn = Self::regex_parser(b);

        an.partial_cmp(&bn).unwrap_or(Ordering::Equal)
    }

    /// Sorts paths by volume and chapter numbers in filenames
    /// Expects filenames in format "volume-chapter" (e.g., "1-15.jpg")
    ///
    /// # Arguments
    ///
    /// * `a` - First path to compare
    /// * `b` - Second path to compare
    ///
    /// # Returns
    ///
    /// * `Ordering` - Ordering based on volume then chapter
    pub fn sort_by_name_volume_chapter(a: &PathBuf, b: &PathBuf) -> Ordering {
        // Cache the parsed numbers for better performance
        fn parse_numbers(path: &PathBuf) -> (Option<f64>, Option<f64>) {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                let parts: Vec<&str> = file_name.split("-").collect();
                let first = parts.first().and_then(|s| s.parse::<f64>().ok());
                let last = parts.last().and_then(|s| s.parse::<f64>().ok());
                (first, last)
            } else {
                (None, None)
            }
        }

        let (a_vol, a_chap) = parse_numbers(a);
        let (b_vol, b_chap) = parse_numbers(b);

        // This will compare the volume number first and then the chapter number
        match a_vol.partial_cmp(&b_vol) {
            Some(Ordering::Equal) => a_chap.partial_cmp(&b_chap).unwrap_or(Ordering::Equal),
            Some(order) => order,
            None => Ordering::Equal,
        }
    }
}
