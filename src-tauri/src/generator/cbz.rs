use crate::generator::Generator;
use crate::prelude::*;
use async_trait::async_trait;
use log::{debug, error, info, trace};
use memmap2::MmapOptions;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tauri::async_runtime::spawn_blocking;
use tokio::fs;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

/// A generator for creating CBZ (Comic Book ZIP) files.
///
/// This struct implements the `Generator` trait to package images into
/// a properly formatted CBZ archive with optional metadata.
pub struct Cbz {
    /// The ZIP writer for archive creation
    zip: Option<ZipWriter<File>>,
    /// Options for image files (no compression - images are already compressed)
    image_options: SimpleFileOptions,
    /// Options for metadata files (high compression for text)
    metadata_options: SimpleFileOptions,
    /// Current page index for sequential numbering
    page_index: usize,
}

#[async_trait(?Send)]
impl Generator for Cbz {
    /// Creates a new CBZ generator with the specified output path and filename.
    ///
    /// # Parameters
    /// * `output_path` - The directory where the CBZ file will be created
    /// * `filename` - The name of the CBZ file (without extension)
    ///
    /// # Returns
    /// A Result containing a new Cbz instance or an Error if creation fails.
    fn new(output_path: &str, filename: &str) -> Result<Self, Error> {
        info!(
            "Creating new CBZ generator: output_path={}, filename={}",
            output_path, filename
        );

        // Use Stored (no compression) for images since they're already compressed
        // This significantly reduces CPU usage and often produces smaller files
        // because compressed images don't compress further and add overhead
        let image_options: SimpleFileOptions = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Stored)
            .unix_permissions(0o755);

        // Use maximum compression for metadata XML files
        let metadata_options: SimpleFileOptions = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .compression_level(Some(9))
            .unix_permissions(0o755);

        debug!("CBZ image options: compression=Stored (no recompression), permissions=0o755");
        debug!("CBZ metadata options: compression=Deflated level 9, permissions=0o755");

        let output_file = format!("{}/{}.cbz", output_path, filename);
        debug!("Creating CBZ file at: {}", output_file);

        let file = match File::create(&output_file) {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to create CBZ file {}: {}", output_file, e);
                return Err(Error::from(e));
            }
        };

        let zip = ZipWriter::new(file);
        debug!("ZipWriter initialized successfully");

        Ok(Cbz {
            zip: Some(zip),
            image_options,
            metadata_options,
            page_index: 0,
        })
    }

    /// Adds an image to the CBZ file as a sequentially numbered page.
    ///
    /// Images are added with filenames like "page_001.ext", "page_002.ext", etc.
    /// Uses memory mapping for efficient file handling.
    ///
    /// # Parameters
    /// * `image_path` - Path to the image file to add
    ///
    /// # Returns
    /// A Result containing a reference to self for method chaining or an Error.
    async fn add_page(&mut self, image_path: &PathBuf) -> Result<&mut Self, Error> {
        info!("Adding page {} from: {:?}", self.page_index + 1, image_path);

        let (image_extension, _) = match get_file_info(image_path) {
            Ok(info) => info,
            Err(e) => {
                error!("Failed to get file info for image {:?}: {}", image_path, e);
                return Err(e);
            }
        };
        trace!("Image extension: {}", image_extension);

        // Open the file
        trace!("Opening image file: {:?}", image_path);
        let file = match fs::File::open(image_path).await {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to open image file {:?}: {}", image_path, e);
                return Err(Error::from(e));
            }
        };

        let file_std = file.into_std().await;
        let options = self.image_options;
        let file_name = format!("page_{:03}.{}", self.page_index + 1, image_extension);
        debug!(
            "Adding to CBZ as: {} (stored without recompression)",
            file_name
        );

        let zip = match self.zip.as_mut() {
            Some(z) => z,
            None => {
                error!("Zip writer not available");
                return Err(Error::Unsupported("Zip writer not available".to_string()));
            }
        };

        // Create the read-only memory map
        trace!("Creating memory map for: {:?}", image_path);
        let mmap = match unsafe { MmapOptions::new().map(&file_std) } {
            Ok(map) => map,
            Err(e) => {
                error!("Memory mapping failed for {:?}: {}", image_path, e);
                return Err(Error::from(e));
            }
        };

        // Add to zip
        trace!("Starting file entry in ZIP: {}", file_name);
        if let Err(e) = zip.start_file(file_name.clone(), options) {
            error!("Failed to start file entry {}: {}", file_name, e);
            return Err(Error::from(e));
        }

        trace!("Writing {} bytes to ZIP", mmap.len());
        if let Err(e) = zip.write_all(&mmap[..]) {
            error!("Failed to write file data for {}: {}", file_name, e);
            return Err(Error::from(e));
        }

        // Increment page index
        self.page_index += 1;
        debug!("Page added successfully. Total pages: {}", self.page_index);

        Ok(self)
    }

    /// Sets metadata for the CBZ file by adding a ComicInfo.xml file.
    ///
    /// Uses a template XML file to create standardized comic metadata.
    ///
    /// # Parameters
    /// * `title` - The title of the comic
    /// * `volume` - The volume number
    ///
    /// # Returns
    /// A Result containing a reference to self for method chaining or an Error.
    async fn set_metadata(&mut self, title: &str, volume: usize) -> Result<&mut Self, Error> {
        info!("Setting CBZ metadata: title='{}', volume={}", title, volume);
        const TEMPLATE: &str = include_str!("../../templates/template.xml");

        // Create owned copies of any borrowed data
        let title = title.to_string();
        let page_index = self.page_index;
        debug!("Preparing ComicInfo.xml with {} pages", page_index);

        let xml = match spawn_blocking(move || {
            TEMPLATE
                .replace("%title%", &title)
                .replace("%volume%", &volume.to_string())
                .replace("%pagecount%", &page_index.to_string())
        })
        .await
        {
            Ok(content) => content,
            Err(e) => {
                error!("Failed to generate ComicInfo.xml content: {}", e);
                return Err(Error::AsyncTaskError(e.to_string()));
            }
        };

        // Get the zip writer
        let zip = match self.zip.as_mut() {
            Some(z) => z,
            None => {
                error!("Zip writer not available for adding metadata");
                return Err(Error::Unsupported("Zip writer not available".to_string()));
            }
        };

        // Add the metadata file to zip with high compression
        trace!("Adding ComicInfo.xml to CBZ with Deflate compression");
        if let Err(e) = zip.start_file("ComicInfo.xml", self.metadata_options) {
            error!("Failed to start ComicInfo.xml entry: {}", e);
            return Err(Error::from(e));
        }

        if let Err(e) = zip.write_all(xml.as_bytes()) {
            error!("Failed to write ComicInfo.xml data: {}", e);
            return Err(Error::from(e));
        }

        debug!("ComicInfo.xml added successfully");
        Ok(self)
    }

    /// Finalizes and saves the CBZ file.
    ///
    /// This method consumes the Cbz instance and completes the ZIP archive
    /// in a separate blocking task to avoid blocking the async runtime.
    ///
    /// # Returns
    /// A Result indicating success or an Error if saving fails.
    async fn save(mut self) -> Result<(), Error> {
        info!("Finalizing and saving CBZ file");

        // Take ownership of the zip writer
        let zip = match self.zip.take() {
            Some(z) => z,
            None => {
                error!("Zip writer not available for saving");
                return Err(Error::Unsupported("Zip writer not available".to_string()));
            }
        };

        // Finish writing the zip file in a blocking task
        debug!("Finishing ZIP archive in blocking task");
        match spawn_blocking(move || match zip.finish() {
            Ok(_) => {
                trace!("ZIP archive finalized successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to finalize ZIP archive: {}", e);
                Err(Error::from(e))
            }
        })
        .await
        {
            Ok(result) => match result {
                Ok(_) => {
                    info!("CBZ file saved successfully");
                    Ok(())
                }
                Err(e) => {
                    error!("Error while finalizing CBZ: {}", e);
                    Err(e)
                }
            },
            Err(e) => {
                error!("Async task error while saving CBZ: {}", e);
                Err(Error::AsyncTaskError(e.to_string()))
            }
        }
    }
}
