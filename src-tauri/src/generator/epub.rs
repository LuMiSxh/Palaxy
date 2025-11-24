use std::fs::File;
use std::io::{BufWriter, Cursor};
use std::path::{Path, PathBuf};

use crate::generator::Generator;
use crate::prelude::*;
use epub_builder::{EpubBuilder, EpubContent, EpubVersion, ZipLibrary};
use log::{debug, error, info, trace};

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
    /// Count of pages added to the EPUB
    page_count: usize,
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

        // Pre-allocate string capacity for cover path
        let mut cover_path = String::with_capacity(16 + cover_extension.len());
        use std::fmt::Write;
        write!(&mut cover_path, "data/cover.{}", cover_extension)
            .expect("String write cannot fail");

        match self
            .epub
            .add_cover_image(cover_path, cover_file, cover_mime)
        {
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
}

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
            output_path: output_path.into(),
            filename: filename.into(),
            reading_direction: None,
            page_count: 0,
        })
    }

    fn add_page_from_memory(&mut self, data: &[u8], extension: &str) -> EResult<&mut Self> {
        self.page_count += 1;

        let mime = match extension.to_lowercase().as_str() {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "webp" => "image/webp",
            "avif" => "image/avif",
            _ => "application/octet-stream",
        };

        let name_stem = format!("page_{:03}", self.page_count);
        let image_filename = format!("images/{}.{}", name_stem, extension);
        let content_filename = format!("{}.xhtml", name_stem);

        trace!("Adding EPUB page {}: {}", self.page_count, image_filename);

        // Cursor<Vec<u8>> implements Read, which epub-builder accepts
        if let Err(e) = self
            .epub
            .add_resource(&image_filename, Cursor::new(data), mime)
        {
            error!("Failed to add EPUB resource {}: {}", image_filename, e);
            return Err(Error::from(e));
        }

        let xhtml = generate_xhtml(&image_filename)?;

        if let Err(e) = self
            .epub
            .add_content(EpubContent::new(content_filename, xhtml.as_bytes()))
        {
            error!("Failed to add EPUB content: {}", e);
            return Err(Error::from(e));
        }

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
    fn set_metadata(&mut self, title: &str, volume: usize) -> EResult<&mut Self> {
        info!(
            "Setting EPUB metadata: title='{}', volume={}",
            title, volume
        );

        let mut full_title = String::with_capacity(title.len() + 12);
        use std::fmt::Write;
        write!(&mut full_title, "{} | {}", title, volume).expect("String write cannot fail");
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
    fn save(self) -> EResult<()> {
        let output_path = Path::new(&self.output_path);
        let mut epub_filename = String::with_capacity(self.filename.len() + 5);
        use std::fmt::Write;
        write!(&mut epub_filename, "{}.epub", self.filename).expect("String write cannot fail");
        let output_file_path = output_path.join(epub_filename);
        info!("Saving EPUB to: {:?}", output_file_path);

        let file = match File::create(&output_file_path) {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to create output file: {}", e);
                return Err(Error::from(e));
            }
        };

        let buf_writer = BufWriter::with_capacity(64 * 1024, file);

        match self.epub.generate(buf_writer) {
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
