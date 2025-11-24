use crate::generator::Generator;
use crate::prelude::*;
use log::{debug, error, info, trace};
use std::fs::File;
use std::io::{BufWriter, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

/// A generator for creating CBZ (Comic Book ZIP) files.
///
/// This struct implements the `Generator` trait to package images into
/// a properly formatted CBZ archive with optional metadata.
pub struct Cbz {
    /// The ZIP writer for archive creation
    zip: Option<ZipWriter<BufWriter<File>>>,
    /// Options for image files (no compression - images are already compressed)
    image_options: SimpleFileOptions,
    /// Options for metadata files (high compression for text)
    metadata_options: SimpleFileOptions,
    /// Current page index for sequential numbering
    page_index: usize,
}

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

        let buf_writer = BufWriter::with_capacity(64 * 1024, file);
        let zip = ZipWriter::new(buf_writer);
        debug!("ZipWriter initialized successfully");

        Ok(Cbz {
            zip: Some(zip),
            image_options,
            metadata_options,
            page_index: 0,
        })
    }

    /// Adds an image to the CBZ file from in-memory data.
    ///
    /// Images are added with filenames like "page_001.ext", "page_002.ext", etc.
    ///
    /// # Parameters
    /// * `data` - Byte slice containing the image data
    /// * `extension` - File extension indicating the image format (e.g., "jpg", "png")
    /// # Returns
    /// A Result containing a reference to self for method chaining or an Error.
    fn add_page_from_memory(&mut self, data: &[u8], extension: &str) -> Result<&mut Self, Error> {
        // Pre-allocate to avoid allocation on every page
        let mut file_name = String::with_capacity(16);
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

        // 3. Write Data
        if let Err(e) = zip.write_all(data) {
            error!("Failed to write file data: {}", e);
            return Err(Error::from(e));
        }

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
    fn set_metadata(&mut self, title: &str, volume: usize) -> Result<&mut Self, Error> {
        info!("Setting CBZ metadata: title='{}', volume={}", title, volume);
        const TEMPLATE: &str = include_str!("../../templates/template.xml");

        debug!("Preparing ComicInfo.xml with {} pages", self.page_index);

        // Pre-allocate buffer for XML (~500 chars typical)
        let mut volume_str = String::with_capacity(8);
        let mut page_count_str = String::with_capacity(8);
        use std::fmt::Write;
        write!(&mut volume_str, "{}", volume).expect("String write cannot fail");
        write!(&mut page_count_str, "{}", self.page_index).expect("String write cannot fail");

        let xml = TEMPLATE
            .replace("%title%", title)
            .replace("%volume%", &volume_str)
            .replace("%pagecount%", &page_count_str);

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
    ///
    /// This method consumes the Cbz instance and completes the ZIP archive
    /// in a separate blocking task to avoid blocking the async runtime.
    ///
    /// # Returns
    /// A Result indicating success or an Error if saving fails.
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
