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

pub struct Cbz {
    zip: Option<ZipWriter<File>>,
    options: SimpleFileOptions,
    page_index: usize,
}

#[async_trait]
impl Generator for Cbz {
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
