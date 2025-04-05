use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tauri::async_runtime::spawn_blocking;
use tokio::fs;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::generator::Generator;
use crate::prelude::*;
use async_trait::async_trait;
use memmap2::MmapOptions;

/// A generator for creating CBZ (Comic Book ZIP) files.
///
/// This struct implements the `Generator` trait to package images into
/// a properly formatted CBZ archive with optional metadata.
pub struct Cbz {
    /// The ZIP writer for archive creation
    zip: Option<ZipWriter<File>>,
    /// Options for files added to the ZIP archive
    options: SimpleFileOptions,
    /// Current page index for sequential numbering
    page_index: usize,
}

#[async_trait]
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
        let options: SimpleFileOptions = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o755);

        let file = File::create(format!("{}/{}.cbz", output_path, filename))?;
        let zip = ZipWriter::new(file);

        Ok(Cbz {
            zip: Some(zip),
            options,
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
        let (image_extension, _) = get_file_info(image_path)?;

        // Open the file
        let file = fs::File::open(image_path).await?;
        let file_std = file.into_std().await;

        // Create the memory map and add to zip in a blocking task
        let options = self.options;
        let file_name = format!("page_{:03}.{}", self.page_index + 1, image_extension);
        let zip = self
            .zip
            .as_mut()
            .ok_or(Error::Unsupported("Zip writer not available".to_string()))?;

        // Create the read-only memory map
        let mmap = unsafe { MmapOptions::new().map(&file_std)? };

        // Add to zip
        zip.start_file(file_name, options)?;
        zip.write_all(&mmap[..])?;

        // Increment page index
        self.page_index += 1;

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
        const TEMPLATE: &str = include_str!("../../templates/template.xml");

        // Create owned copies of any borrowed data
        let title = title.to_string(); // Convert &str to owned String
        let page_index = self.page_index;

        let xml = spawn_blocking(move || {
            TEMPLATE
                .replace("%title%", &title)
                .replace("%volume%", &volume.to_string())
                .replace("%pagecount%", &page_index.to_string())
        })
        .await
        .map_err(|e| Error::AsyncTaskError(e.to_string()))?;

        // Get the zip writer
        let zip = self
            .zip
            .as_mut()
            .ok_or(Error::Unsupported("Zip writer not available".to_string()))?;

        // Add the metadata file to zip
        zip.start_file("ComicInfo.xml", self.options)?;
        zip.write_all(xml.as_bytes())?;

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
        // Take ownership of the zip writer
        let zip = self
            .zip
            .take()
            .ok_or(Error::Unsupported("Zip writer not available".to_string()))?;

        // Finish writing the zip file in a blocking task to avoid blocking the async runtime
        spawn_blocking(move || {
            zip.finish()?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::AsyncTaskError(e.to_string()))?
    }
}
