//! Directory and file collection with support for ZIP input, flat structures,
//! and arbitrary nesting depth.

use std::path::{Path, PathBuf};

use log::{debug, info, trace, warn};
use rayon::prelude::*;
use tempfile::TempDir;

use common::prelude::*;
use common::utils::is_image_file;

use crate::sorting::natural_sort;

/// Result of discovering the chapter/page structure of an input source.
#[derive(Debug)]
pub struct DiscoverResult {
    /// Each inner Vec is one "chapter" — a leaf directory's sorted image paths.
    /// For flat input, there is exactly one inner Vec.
    pub chapters: Vec<Vec<PathBuf>>,
    /// Non-image files encountered during scanning (for warning at analysis time).
    pub skipped_files: Vec<PathBuf>,
    /// Whether the input was a flat structure (images directly at root, no subdirs with images).
    pub is_flat: bool,
}

/// Manages collection of image files from various input sources.
///
/// Supports directories and ZIP files. For ZIP files, extracts to a temporary
/// directory that is kept alive as long as the Collector exists.
pub struct Collector {
    /// Root directory to scan (either the original directory or extracted temp dir).
    base_directory: PathBuf,
    /// Temporary directory handle for ZIP extraction. Dropped when Collector is dropped.
    temp_dir: Option<TempDir>,
}

impl Collector {
    /// Creates a collector from a path, which may be a directory or ZIP file.
    ///
    /// If the path points to a `.zip` file, it will be extracted to a temporary
    /// directory. The temp directory is kept alive for the lifetime of this Collector.
    pub fn from_path(path: &Path) -> EResult<Self> {
        if path.is_file() && Self::is_zip_file(path) {
            info!("Source is a ZIP file, extracting: {:?}", path);
            let (extracted_path, temp_dir) = Self::extract_zip(path)?;
            info!("Extracted ZIP to: {:?}", extracted_path);
            Ok(Self {
                base_directory: extracted_path,
                temp_dir: Some(temp_dir),
            })
        } else if path.is_dir() {
            info!("Source is a directory: {:?}", path);
            Ok(Self {
                base_directory: path.to_path_buf(),
                temp_dir: None,
            })
        } else {
            Err(Error::InvalidPath(
                path.to_path_buf(),
                "Path is neither a directory nor a ZIP file".into(),
            ))
        }
    }

    /// Takes ownership of the temp directory handle.
    ///
    /// After calling this, the Collector no longer owns the temp dir.
    /// The caller is responsible for keeping the TempDir alive as needed.
    pub fn take_temp_dir(&mut self) -> Option<TempDir> {
        self.temp_dir.take()
    }

    /// Returns true if this collector was created from a ZIP file.
    pub fn is_zip_source(&self) -> bool {
        self.temp_dir.is_some()
    }

    /// Returns the base directory being scanned.
    pub fn base_directory(&self) -> &Path {
        &self.base_directory
    }

    /// Discovers the complete chapter/page structure by recursively scanning
    /// the base directory.
    ///
    /// The algorithm:
    /// 1. Recursively finds all "leaf image directories" — directories that contain
    ///    image files.
    /// 2. If a directory has both images and subdirectories containing images,
    ///    the loose images are treated as their own chapter.
    /// 3. If the root contains only images (no subdirs with images), the result
    ///    is a single chapter with `is_flat = true`.
    /// 4. Non-image files are collected into `skipped_files` for warning reporting.
    pub fn discover(&self) -> EResult<DiscoverResult> {
        info!("Discovering structure in: {:?}", self.base_directory);

        let mut skipped_files = Vec::new();
        let mut leaf_dirs = Self::find_leaf_image_dirs(&self.base_directory, &mut skipped_files)?;

        if leaf_dirs.is_empty() {
            warn!("No image files found in source");
            return Ok(DiscoverResult {
                chapters: Vec::new(),
                skipped_files,
                is_flat: false,
            });
        }

        // Determine if this is a flat structure:
        // flat = only one leaf dir and it IS the base directory
        let is_flat = leaf_dirs.len() == 1 && leaf_dirs[0].0 == self.base_directory;

        // Sort chapters by directory path using natural sort
        leaf_dirs.par_sort_unstable_by(|a, b| natural_sort_by_dir_path(&a.0, &b.0));

        // Sort images within each chapter
        let chapters: Vec<Vec<PathBuf>> = leaf_dirs
            .into_iter()
            .map(|(dir_path, mut images)| {
                debug!(
                    "Chapter {:?}: {} images",
                    dir_path.file_name().unwrap_or_default(),
                    images.len()
                );
                images.par_sort_unstable_by(natural_sort);
                images
            })
            .collect();

        info!(
            "Discovered {} chapters, {} skipped files, flat={}",
            chapters.len(),
            skipped_files.len(),
            is_flat
        );

        Ok(DiscoverResult {
            chapters,
            skipped_files,
            is_flat,
        })
    }

    /// Recursively finds leaf directories that contain image files.
    fn find_leaf_image_dirs(
        dir: &Path,
        skipped_files: &mut Vec<PathBuf>,
    ) -> EResult<Vec<(PathBuf, Vec<PathBuf>)>> {
        let entries: Vec<_> = match std::fs::read_dir(dir) {
            Ok(reader) => reader
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    // Skip hidden files/directories
                    entry
                        .file_name()
                        .to_str()
                        .map(|name| !name.starts_with('.'))
                        .unwrap_or(false)
                })
                .collect(),
            Err(e) => {
                warn!("Failed to read directory {:?}: {}", dir, e);
                return Err(Error::Io(e));
            }
        };

        let mut sub_dirs = Vec::new();
        let mut image_files = Vec::new();

        for entry in entries {
            let path = entry.path();
            let file_type = entry.file_type().map_err(Error::Io)?;

            if file_type.is_dir() {
                sub_dirs.push(path);
            } else if file_type.is_file() {
                if is_image_file(&path) {
                    image_files.push(path);
                } else {
                    trace!("Skipping non-image file: {:?}", path);
                    skipped_files.push(path);
                }
            }
        }

        // If no subdirectories, this is a leaf directory
        if sub_dirs.is_empty() {
            if image_files.is_empty() {
                return Ok(Vec::new());
            }
            return Ok(vec![(dir.to_path_buf(), image_files)]);
        }

        // Recurse into subdirectories
        let mut results = Vec::new();
        for sub_dir in sub_dirs {
            let sub_results = Self::find_leaf_image_dirs(&sub_dir, skipped_files)?;
            results.extend(sub_results);
        }

        // If this directory also has loose images alongside subdirs,
        // treat those images as their own chapter
        if !image_files.is_empty() {
            debug!(
                "Directory {:?} has {} loose images alongside subdirectories",
                dir,
                image_files.len()
            );
            results.push((dir.to_path_buf(), image_files));
        }

        Ok(results)
    }

    /// Checks if a path appears to be a ZIP file.
    fn is_zip_file(path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("zip"))
            .unwrap_or(false)
    }

    /// Extracts a ZIP file to a temporary directory.
    fn extract_zip(zip_path: &Path) -> EResult<(PathBuf, TempDir)> {
        let file = std::fs::File::open(zip_path).map_err(Error::Io)?;
        let mut archive = zip::ZipArchive::new(file)?;

        let temp_dir = tempfile::tempdir().map_err(Error::Io)?;
        debug!(
            "Extracting {} entries to {:?}",
            archive.len(),
            temp_dir.path()
        );

        archive.extract(temp_dir.path())?;

        info!(
            "Successfully extracted {} entries from ZIP",
            archive.len()
        );
        Ok((temp_dir.path().to_path_buf(), temp_dir))
    }
}

/// Natural sort comparison for directory paths by their name component.
fn natural_sort_by_dir_path(a: &Path, b: &Path) -> std::cmp::Ordering {
    let a_name = a.file_name().and_then(|s| s.to_str()).unwrap_or("");
    let b_name = b.file_name().and_then(|s| s.to_str()).unwrap_or("");

    crate::sorting::natural_sort_by_name(&PathBuf::from(a_name), &PathBuf::from(b_name))
}
