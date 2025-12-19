//! Directory and file collection functionality.

use std::cmp::Ordering;
use std::path::PathBuf;
use std::sync::Arc;

use log::{debug, error, info, trace, warn};
use rayon::prelude::*;
use tauri::async_runtime::{spawn, JoinHandle};
use tokio::fs::{read_dir, DirEntry};
use tokio::sync::Semaphore;

use common::prelude::*;

use crate::constants::MAX_CONCURRENT_DIRS;

/// Manages collection and organization of image files in a directory structure.
#[derive(Debug, Clone)]
pub struct Collector {
    /// Root directory containing chapters or volumes.
    base_directory: PathBuf,
}

impl Collector {
    /// Creates a new collector instance for the specified directory.
    #[inline]
    pub fn new(base_directory: &PathBuf) -> Self {
        info!("Creating new Collector for directory: {:?}", base_directory);
        Self {
            base_directory: base_directory.clone(),
        }
    }

    /// Collects chapter directories from the base directory.
    pub async fn collect_chapters(
        &mut self,
        comparator: Option<&'static (dyn Fn(&PathBuf, &PathBuf) -> Ordering + Sync)>,
    ) -> EResult<Vec<PathBuf>> {
        info!("Collecting chapters from {:?}", self.base_directory);
        let mut chapters = Self::collect_parallel(&self.base_directory, true).await?;
        debug!("Found {} potential chapter directories", chapters.len());

        if let Some(comparator) = comparator {
            debug!("Sorting chapters using custom comparator");
            chapters.par_sort_unstable_by(comparator);
        }

        info!("Collected {} chapters", chapters.len());
        Ok(chapters)
    }

    /// Collects page images from each chapter directory.
    pub async fn collect_pages(
        &self,
        chapters: Vec<PathBuf>,
        comparator: Option<&'static (dyn Fn(&PathBuf, &PathBuf) -> Ordering + Sync)>,
    ) -> EResult<Vec<Vec<PathBuf>>> {
        info!("Collecting pages from {} chapters", chapters.len());
        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_DIRS));
        debug!(
            "Using semaphore with {} permits for concurrent operations",
            MAX_CONCURRENT_DIRS
        );

        // Pre-allocate result vector with exact capacity
        let chapter_count = chapters.len();
        let pages = Arc::new(tokio::sync::Mutex::new(vec![Vec::new(); chapter_count]));

        let handles: Vec<JoinHandle<EResult<()>>> = chapters
            .into_par_iter()
            .enumerate()
            .map(|(index, chapter_dir)| {
                let semaphore = Arc::clone(&semaphore);
                let pages = Arc::clone(&pages);
                trace!("Spawning task for chapter directory: {:?}", chapter_dir);

                spawn(async move {
                    let _permit = semaphore.acquire().await.map_err(|e| {
                        error!("Failed to acquire semaphore: {}", e);
                        Error::AsyncTaskError(format!("Failed to acquire semaphore: {}", e))
                    })?;

                    debug!("Processing chapter directory {}: {:?}", index, chapter_dir);
                    let mut chapter_images = Self::collect_parallel(&chapter_dir, false).await?;
                    trace!("Found {} images in chapter {}", chapter_images.len(), index);

                    if let Some(comparator) = comparator {
                        trace!("Sorting images in chapter {}", index);
                        chapter_images.par_sort_unstable_by(comparator);
                    }

                    // Update the pages vector directly
                    pages.lock().await[index] = chapter_images;

                    Ok(())
                })
            })
            .collect();

        // Wait for all tasks to complete
        for handle in handles {
            handle
                .await
                .map_err(|e| Error::AsyncTaskError(e.to_string()))??;
        }

        // Extract final result
        let result = Arc::try_unwrap(pages)
            .map(|mutex| mutex.into_inner())
            .unwrap_or_else(|arc| (*arc.blocking_lock()).clone());

        info!("Collected pages for {} chapters", result.len());
        Ok(result)
    }

    /// Collects directory contents in parallel with filtering options.
    pub async fn collect_parallel(directory: &PathBuf, only_dirs: bool) -> EResult<Vec<PathBuf>> {
        debug!(
            "Collecting {} from directory: {:?}",
            if only_dirs { "directories" } else { "files" },
            directory
        );

        let mut dir_reader = match read_dir(directory).await {
            Ok(reader) => reader,
            Err(e) => {
                error!("Failed to read directory {:?}: {}", directory, e);
                return Err(Error::Io(e));
            }
        };

        // Collect entries into a buffer to reduce async overhead
        let mut entries = Vec::with_capacity(64); // Reasonable default capacity

        loop {
            match dir_reader.next_entry().await {
                Ok(Some(entry)) => {
                    if let Some(path) = Self::process_entry(entry, only_dirs).await? {
                        entries.push(path);
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    error!("Error reading entry in {:?}: {}", directory, e);
                    return Err(Error::Io(e));
                }
            }
        }

        info!("Collected {} entries from {:?}", entries.len(), directory);
        Ok(entries)
    }

    /// Processes a single directory entry and validates it.
    #[inline]
    async fn process_entry(entry: DirEntry, only_dirs: bool) -> EResult<Option<PathBuf>> {
        let path = entry.path();

        // Skip hidden files (early return for better performance)
        if let Some(file_name) = path.file_name() {
            let name = file_name.to_string_lossy();
            if name.starts_with('.') {
                trace!("Skipping hidden file: {:?}", path);
                return Ok(None);
            }
        }

        // Apply directory/file filter
        let metadata = entry.metadata().await.map_err(Error::Io)?;
        let is_dir = metadata.is_dir();

        if (only_dirs && !is_dir) || (!only_dirs && is_dir) {
            warn!(
                "Expected {}, got {}: {:?}",
                if only_dirs { "directory" } else { "file" },
                if is_dir { "directory" } else { "file" },
                path
            );
            return Err(Error::InvalidPath(
                path.clone(),
                format!(
                    "{} expected, got {}",
                    if only_dirs { "Directory" } else { "File" },
                    if is_dir { "directory" } else { "file" }
                ),
            ));
        }

        trace!("Adding path: {:?}", path);
        Ok(Some(path))
    }

    /// Filters paths based on a test condition.
    #[inline]
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
}
