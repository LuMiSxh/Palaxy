use std::fs::File;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use crate::generator::Generator;
use crate::prelude::*;
use async_trait::async_trait;
use epub_builder::{EpubBuilder, EpubContent, EpubVersion, ZipLibrary};
use log::{debug, error, info, trace};
use memmap2::MmapOptions;

/// Generates XHTML content for an image to be included in the EPUB.
///
/// # Arguments
///
/// * `image_source` - Path to the image file relative to the EPUB root
///
/// # Returns
///
/// * `EResult<String>` - The generated XHTML content or an error
fn generate_xhtml(image_source: &str) -> EResult<String> {
    trace!("Generating XHTML content for image: {}", image_source);
    const TEMPLATE: &str = include_str!("../../templates/template.xhtml");
    let xhtml = TEMPLATE
        .replace("%title%", image_source)
        .replace("%src%", image_source)
        .replace("%alt%", image_source);
    debug!("XHTML template substitution completed for {}", image_source);
    Ok(xhtml)
}

/// A generator for creating EPUB files with images.
///
/// This struct wraps the `EpubBuilder` functionality and implements the `Generator` trait
/// to provide a standardized interface for creating EPUB documents with images.
pub struct EPub {
    /// The underlying EPUB builder
    epub: EpubBuilder<ZipLibrary>,
    /// Directory where the EPUB file will be saved
    output_path: String,
    /// Name of the output file (without extension)
    filename: String,
    /// Reading direction for the EPUB content
    reading_direction: Option<Direction>,
}

impl EPub {
    /// Sets custom metadata in the EPUB file.
    ///
    /// # Arguments
    ///
    /// * `key` - Metadata key
    /// * `value` - Metadata value
    ///
    /// # Returns
    ///
    /// * `EResult<&mut Self>` - Self reference for method chaining or an error
    pub fn set_custom_metadata(&mut self, key: &str, value: &str) -> EResult<&mut Self> {
        debug!("Setting EPUB metadata: {}={}", key, value);
        self.epub.metadata(key, value)?;
        Ok(self)
    }

    /// Sets the cover image for the EPUB file.
    ///
    /// # Arguments
    ///
    /// * `cover_image_path` - Path to the cover image file
    ///
    /// # Returns
    ///
    /// * `EResult<&mut Self>` - Self reference for method chaining or an error
    pub fn set_cover(&mut self, cover_image_path: &PathBuf) -> EResult<&mut Self> {
        info!("Setting EPUB cover image: {:?}", cover_image_path);
        let (cover_extension, cover_mime) = get_file_info(cover_image_path)?;
        debug!(
            "Cover image info: extension={}, mime={}",
            cover_extension, cover_mime
        );

        let cover_file = match File::open(cover_image_path) {
            Ok(file) => file,
            Err(e) => {
                error!(
                    "Failed to open cover image file {:?}: {}",
                    cover_image_path, e
                );
                return Err(Error::from(e));
            }
        };

        match self.epub.add_cover_image(
            format!("data/cover.{}", cover_extension),
            cover_file,
            cover_mime,
        ) {
            Ok(_) => {
                debug!("Cover image added successfully");
                Ok(self)
            }
            Err(e) => {
                error!("Failed to add cover image: {}", e);
                Err(Error::from(e))
            }
        }
    }

    /// Sets the language for the EPUB file.
    ///
    /// # Arguments
    ///
    /// * `lang` - Language code (e.g., "en", "ja")
    ///
    /// # Returns
    ///
    /// * `EResult<&mut Self>` - Self reference for method chaining or an error
    pub fn set_lang(&mut self, lang: &str) -> EResult<&mut Self> {
        info!("Setting EPUB language to: {}", lang);
        self.epub.set_lang(lang);
        Ok(self)
    }

    /// Sets the reading direction for the EPUB content.
    ///
    /// # Arguments
    ///
    /// * `direction` - Reading direction (LTR or RTL)
    ///
    /// # Returns
    ///
    /// * `&mut Self` - Self reference for method chaining
    pub fn set_reading_direction(&mut self, direction: Direction) -> &mut Self {
        info!("Setting EPUB reading direction to: {:?}", direction);
        self.reading_direction = Some(direction);
        self
    }

    /// Adds a chapter containing multiple image pages to the EPUB.
    ///
    /// # Arguments
    ///
    /// * `chapter_count` - Chapter number/index
    /// * `image_paths` - Vector of paths to the images in this chapter
    ///
    /// # Returns
    ///
    /// * `EResult<&mut Self>` - Self reference for method chaining or an error
    pub async fn add_chapter(
        &mut self,
        chapter_count: usize,
        image_paths: &Vec<PathBuf>,
    ) -> EResult<&mut Self> {
        info!(
            "Adding chapter {} with {} images",
            chapter_count,
            image_paths.len()
        );

        for (i, path) in image_paths.iter().enumerate() {
            trace!(
                "Processing image {}/{} at path: {:?}",
                i + 1,
                image_paths.len(),
                path
            );

            let image_file = match File::open(&path) {
                Ok(file) => file,
                Err(e) => {
                    error!("Failed to open image file {:?}: {}", path, e);
                    return Err(Error::from(e));
                }
            };

            let (image_extension, image_mime) = get_file_info(&path)?;
            debug!(
                "Image info: extension={}, mime={}",
                image_extension, image_mime
            );

            let image_name = format!("images/{}/{}.{}", chapter_count, i + 1, image_extension);
            let image_xhtml = generate_xhtml(&image_name)?;

            trace!("Adding resource: {}", image_name);
            if let Err(e) = self.epub.add_resource(&image_name, image_file, image_mime) {
                error!("Failed to add image resource {}: {}", image_name, e);
                return Err(Error::from(e));
            }

            let content_path = format!("{}-{}.xhtml", chapter_count, i + 1);
            trace!("Adding content: {}", content_path);
            if let Err(e) = self.epub.add_content(EpubContent::new(
                content_path.clone(),
                image_xhtml.as_bytes(),
            )) {
                error!("Failed to add XHTML content {}: {}", content_path, e);
                return Err(Error::from(e));
            }
        }
        debug!("Chapter {} added successfully", chapter_count);
        Ok(self)
    }

    /// Adds a resource to the EPUB using memory mapping for efficient handling of large files.
    ///
    /// # Arguments
    ///
    /// * `resource_path` - Path where the resource will be stored in the EPUB
    /// * `image_path` - Path to the image file on the filesystem
    ///
    /// # Returns
    ///
    /// * `Result<&mut Self, Error>` - Self reference for method chaining or an error
    pub async fn add_resource_mmap(
        &mut self,
        resource_path: &str,
        image_path: &PathBuf,
    ) -> Result<&mut Self, Error> {
        debug!(
            "Adding memory-mapped resource from {:?} as {}",
            image_path, resource_path
        );

        let (_, image_mime) = match get_file_info(image_path) {
            Ok(info) => info,
            Err(e) => {
                error!("Failed to get file info for {:?}: {}", image_path, e);
                return Err(e);
            }
        };

        // Open the file asynchronously
        let file = match tokio::fs::File::open(image_path).await {
            Ok(f) => f,
            Err(e) => {
                error!(
                    "Failed to open file for memory mapping {:?}: {}",
                    image_path, e
                );
                return Err(Error::from(e));
            }
        };

        let file_std = file.into_std().await;
        let epub = &mut self.epub;
        let path = resource_path.to_string();
        let mime = image_mime.to_string();

        trace!("Creating memory map for file: {:?}", image_path);
        let mmap = match unsafe { MmapOptions::new().map(&file_std) } {
            Ok(map) => map,
            Err(e) => {
                error!("Memory mapping failed for {:?}: {}", image_path, e);
                return Err(Error::from(e));
            }
        };

        // Add resource directly from memory-mapped data
        if let Err(e) = epub.add_resource(&path, Cursor::new(&mmap[..]), &mime) {
            error!("Failed to add memory-mapped resource {}: {}", path, e);
            return Err(Error::from(e));
        }

        trace!("Memory-mapped resource added successfully: {}", path);
        Ok(self)
    }
}

#[async_trait]
impl Generator for EPub {
    /// Creates a new EPUB generator.
    ///
    /// # Arguments
    ///
    /// * `output_path` - Directory where the EPUB file will be saved
    /// * `filename` - Name of the output file (without extension)
    ///
    /// # Returns
    ///
    /// * `EResult<Self>` - A new EPub instance or an error
    fn new(output_path: &str, filename: &str) -> EResult<Self> {
        info!(
            "Creating new EPUB generator: output_path={}, filename={}",
            output_path, filename
        );

        let mut epub = match EpubBuilder::new(ZipLibrary::new()?) {
            Ok(builder) => builder,
            Err(e) => {
                error!("Failed to create EPUB builder: {}", e);
                return Err(Error::from(e));
            }
        };

        epub.epub_version(EpubVersion::V30);
        debug!("Setting EPUB version to 3.0");

        if let Err(e) = epub.stylesheet(include_bytes!("../../templates/template.css").as_slice()) {
            error!("Failed to add stylesheet: {}", e);
            return Err(Error::from(e));
        }

        debug!("Stylesheet added successfully");
        Ok(EPub {
            epub,
            output_path: output_path.to_string(),
            filename: filename.to_string(),
            reading_direction: None,
        })
    }

    /// Adds a single image page to the EPUB.
    ///
    /// Note: This is a simplified interface that treats each page as its own chapter
    /// for consistency with the Generator trait.
    ///
    /// # Arguments
    ///
    /// * `image_path` - Path to the image file
    ///
    /// # Returns
    ///
    /// * `EResult<&mut Self>` - Self reference for method chaining or an error
    async fn add_page(&mut self, image_path: &PathBuf) -> EResult<&mut Self> {
        info!("Adding page with image: {:?}", image_path);

        let (image_extension, _) = match get_file_info(&image_path) {
            Ok(info) => info,
            Err(e) => {
                error!("Failed to get file info for page image: {}", e);
                return Err(e);
            }
        };

        // Use the page index as chapter count for this simplified version
        let chapter_count = 1;
        let i = 0;

        let image_name = format!("images/{}/{}.{}", chapter_count, i + 1, image_extension);
        debug!("Using image name: {}", image_name);

        let image_xhtml = match generate_xhtml(&image_name) {
            Ok(xhtml) => xhtml,
            Err(e) => {
                error!("Failed to generate XHTML for page: {}", e);
                return Err(e);
            }
        };

        if let Err(e) = self.add_resource_mmap(&image_name, image_path).await {
            error!("Failed to add page resource: {}", e);
            return Err(e);
        }

        let content_path = format!("{}-{}.xhtml", chapter_count, i + 1);
        if let Err(e) = self.epub.add_content(EpubContent::new(
            content_path.clone(),
            image_xhtml.as_bytes(),
        )) {
            error!("Failed to add page XHTML content: {}", e);
            return Err(Error::from(e));
        }

        debug!("Page added successfully");
        Ok(self)
    }

    /// Sets metadata for the EPUB file, including title and volume number.
    ///
    /// # Arguments
    ///
    /// * `title` - Title of the EPUB
    /// * `volume` - Volume number
    ///
    /// # Returns
    ///
    /// * `EResult<&mut Self>` - Self reference for method chaining or an error
    async fn set_metadata(&mut self, title: &str, volume: usize) -> EResult<&mut Self> {
        info!(
            "Setting EPUB metadata: title='{}', volume={}",
            title, volume
        );

        let full_title = format!("{} | {}", title, volume);
        if let Err(e) = self.epub.metadata("title", &full_title) {
            error!("Failed to set title metadata: {}", e);
            return Err(Error::from(e));
        }

        // If the reading direction is set, include it in metadata
        if let Some(direction) = &self.reading_direction {
            let dir_str = match direction {
                Direction::Ltr => "ltr",
                Direction::Rtl => "rtl",
            };
            debug!("Setting reading direction metadata: {}", dir_str);

            if let Err(e) = self.epub.metadata("direction", dir_str) {
                error!("Failed to set direction metadata: {}", e);
                return Err(Error::from(e));
            }
        }

        debug!("Setting filename to: {}", title);
        self.filename = title.to_string();
        Ok(self)
    }

    /// Finalizes and saves the EPUB file to the specified output path.
    ///
    /// # Returns
    ///
    /// * `EResult<()>` - Success or an error
    async fn save(mut self) -> EResult<()> {
        let output_path = Path::new(&self.output_path);
        let output_file_path = output_path.join(format!("{}.epub", self.filename));
        info!("Saving EPUB to: {:?}", output_file_path);

        let file = match File::create(&output_file_path) {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to create output file: {}", e);
                return Err(Error::from(e));
            }
        };

        match self.epub.generate(file) {
            Ok(_) => {
                info!("EPUB file generated successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to generate EPUB file: {}", e);
                Err(Error::from(e))
            }
        }
    }
}
