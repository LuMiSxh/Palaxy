use crate::generator::Generator;
use common::prelude::*;
use log::{debug, error, info, trace};
use std::fs::File;
use std::io::{BufWriter, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

/// CBZ (Comic Book ZIP) generator.
pub struct Cbz {
    /// ZIP writer for archive creation.
    zip: Option<ZipWriter<BufWriter<File>>>,
    /// Options for image files (no compression).
    image_options: SimpleFileOptions,
    /// Options for metadata files (high compression).
    metadata_options: SimpleFileOptions,
    /// Current page index for sequential numbering.
    page_index: usize,
}

impl Generator for Cbz {
    /// Creates a new CBZ generator.
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

        // Pre-allocate output filename to avoid allocations
        let mut output_file = String::with_capacity(output_path.len() + filename.len() + 6);
        use std::fmt::Write;
        write!(&mut output_file, "{}/{}.cbz", output_path, filename)
            .expect("String write cannot fail");

        debug!("Creating CBZ file at: {}", output_file);

        let file = match File::create(&output_file) {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to create CBZ file {}: {}", output_file, e);
                return Err(Error::from(e));
            }
        };

        // Use larger buffer for better I/O performance
        let buf_writer = BufWriter::with_capacity(128 * 1024, file);
        let zip = ZipWriter::new(buf_writer);
        debug!("ZipWriter initialized successfully");

        Ok(Cbz {
            zip: Some(zip),
            image_options,
            metadata_options,
            page_index: 0,
        })
    }

    /// Adds a page from in-memory image data.
    fn add_page_from_memory(&mut self, data: &[u8], extension: &str) -> Result<&mut Self, Error> {
        // Pre-allocate to avoid allocation on every page
        let mut file_name = String::with_capacity(16 + extension.len());
        use std::fmt::Write;
        write!(
            &mut file_name,
            "page_{:03}.{}",
            self.page_index + 1,
            extension
        )
        .expect("String write cannot fail");

        let zip = match self.zip.as_mut() {
            Some(z) => z,
            None => return Err(Error::Unsupported("Zip writer not available".into())),
        };

        // We use Stored (no compression) because images (AVIF/WebP/JPEG) are already compressed.
        // Re-compressing them is a waste of CPU cycles.
        if let Err(e) = zip.start_file(file_name, self.image_options) {
            error!("Failed to start file entry: {}", e);
            return Err(Error::from(e));
        }

        // Write data in one go
        if let Err(e) = zip.write_all(data) {
            error!("Failed to write file data: {}", e);
            return Err(Error::from(e));
        }

        self.page_index += 1;
        Ok(self)
    }

    /// Sets document metadata by adding ComicInfo.xml.
    fn set_metadata(&mut self, title: &str, volume: usize) -> Result<&mut Self, Error> {
        info!("Setting CBZ metadata: title='{}', volume={}", title, volume);
        const TEMPLATE: &str = include_str!("../../templates/template.xml");

        debug!("Preparing ComicInfo.xml with {} pages", self.page_index);

        let xml = TEMPLATE
            .replace("%title%", title)
            .replace("%volume%", &volume.to_string())
            .replace("%pagecount%", &self.page_index.to_string());

        // Get the zip writer
        let zip = match self.zip.as_mut() {
            Some(z) => z,
            None => {
                error!("Zip writer not available for adding metadata");
                return Err(Error::Unsupported("Zip writer not available".into()));
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
    fn save(mut self) -> Result<(), Error> {
        info!("Finalizing and saving CBZ file");

        // Take ownership of the zip writer
        let zip = match self.zip.take() {
            Some(z) => z,
            None => {
                error!("Zip writer not available for saving");
                return Err(Error::Unsupported("Zip writer not available".into()));
            }
        };

        // Finish writing the zip file
        debug!("Finishing ZIP archive");
        match zip.finish() {
            Ok(_) => {
                trace!("ZIP archive finalized successfully");
                info!("CBZ file saved successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to finalize ZIP archive: {}", e);
                Err(Error::from(e))
            }
        }
    }
}
