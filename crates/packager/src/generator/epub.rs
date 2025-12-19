use std::fs::File;
use std::io::{BufWriter, Cursor};
use std::path::{Path, PathBuf};

use crate::generator::Generator;
use common::prelude::*;
use epub_builder::{EpubBuilder, EpubContent, EpubVersion, ZipLibrary};
use log::{debug, error, info, trace};

/// Generates XHTML content for an image page.
#[inline]
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

/// EPUB generator for comic/manga files.
pub struct EPub {
    /// EPUB builder instance.
    epub: EpubBuilder<ZipLibrary>,
    /// Output directory path.
    output_path: String,
    /// Output filename (without extension).
    filename: String,
    /// Reading direction for content.
    reading_direction: Option<Direction>,
    /// Page count.
    page_count: usize,
}

impl EPub {
    /// Sets custom metadata.
    #[inline]
    pub fn set_custom_metadata(&mut self, key: &str, value: &str) -> EResult<&mut Self> {
        debug!("Setting EPUB metadata: {}={}", key, value);
        self.epub.metadata(key, value)?;
        Ok(self)
    }

    /// Sets the cover image.
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

    /// Sets the language.
    #[inline]
    pub fn set_lang(&mut self, lang: &str) -> EResult<&mut Self> {
        info!("Setting EPUB language to: {}", lang);
        self.epub.set_lang(lang);
        Ok(self)
    }

    /// Sets the reading direction.
    #[inline]
    pub fn set_reading_direction(&mut self, direction: Direction) -> &mut Self {
        info!("Setting EPUB reading direction to: {:?}", direction);
        self.reading_direction = Some(direction);
        self
    }
}

impl Generator for EPub {
    /// Creates a new EPUB generator.
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

        // Use static strings where possible to avoid allocations
        let mime = match extension.to_lowercase().as_str() {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "webp" => "image/webp",
            "avif" => "image/avif",
            _ => "application/octet-stream",
        };

        // Pre-allocate strings with exact capacity
        let mut image_filename = String::with_capacity(20 + extension.len());
        let mut content_filename = String::with_capacity(16);

        use std::fmt::Write;
        write!(
            &mut image_filename,
            "images/page_{:03}.{}",
            self.page_count, extension
        )
        .expect("String write cannot fail");
        write!(&mut content_filename, "page_{:03}.xhtml", self.page_count)
            .expect("String write cannot fail");

        trace!("Adding EPUB page {}: {}", self.page_count, image_filename);

        // Cursor<&[u8]> is more efficient than Cursor<Vec<u8>>
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

    /// Sets document metadata.
    fn set_metadata(&mut self, title: &str, volume: usize) -> EResult<&mut Self> {
        info!(
            "Setting EPUB metadata: title='{}', volume={}",
            title, volume
        );

        // Pre-allocate with exact capacity
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

    /// Finalizes and saves the EPUB file.
    fn save(self) -> EResult<()> {
        let output_path = Path::new(&self.output_path);

        // Pre-allocate filename string
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

        // Use larger buffer for better I/O performance
        let buf_writer = BufWriter::with_capacity(128 * 1024, file);

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
