use std::fs::File;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use crate::generator::Generator;
use crate::prelude::*;
use async_trait::async_trait;
use epub_builder::{EpubBuilder, EpubContent, EpubVersion, ZipLibrary};
use memmap2::MmapOptions;

fn generate_xhtml(image_source: &str) -> EResult<String> {
    const TEMPLATE: &str = include_str!("../../templates/template.xhtml");
    let xhtml = TEMPLATE
        .replace("%title%", image_source)
        .replace("%src%", image_source)
        .replace("%alt%", image_source);
    Ok(xhtml)
}

pub struct EPub {
    epub: EpubBuilder<ZipLibrary>,
    output_path: String,
    filename: String,
    reading_direction: Option<Direction>,
}

impl EPub {
    // Additional helper methods needed for EPUB
    pub fn set_custom_metadata(&mut self, key: &str, value: &str) -> EResult<&mut Self> {
        self.epub.metadata(key, value)?;
        Ok(self)
    }

    pub fn set_cover(&mut self, cover_image_path: &PathBuf) -> EResult<&mut Self> {
        let (cover_extension, cover_mime) = get_file_info(cover_image_path)?;
        let cover_file = File::open(cover_image_path)?;

        self.epub.add_cover_image(
            format!("data/cover.{}", cover_extension),
            cover_file,
            cover_mime,
        )?;
        Ok(self)
    }

    pub fn set_lang(&mut self, lang: &str) -> EResult<&mut Self> {
        self.epub.set_lang(lang);
        Ok(self)
    }

    pub fn set_reading_direction(&mut self, direction: Direction) -> &mut Self {
        self.reading_direction = Some(direction);
        self
    }

    // Add chapter method that wraps multiple pages
    pub async fn add_chapter(
        &mut self,
        chapter_count: usize,
        image_paths: &Vec<PathBuf>,
    ) -> EResult<&mut Self> {
        for (i, path) in image_paths.iter().enumerate() {
            let image_file = File::open(&path)?;
            let (image_extension, image_mime) = get_file_info(&path)?;

            let image_name = format!("images/{}/{}.{}", chapter_count, i + 1, image_extension);
            let image_xhtml = generate_xhtml(&image_name)?;

            self.epub
                .add_resource(&image_name, image_file, image_mime)?;

            self.epub.add_content(EpubContent::new(
                format!("{}-{}.xhtml", chapter_count, i + 1),
                image_xhtml.as_bytes(),
            ))?;
        }
        Ok(self)
    }

    pub async fn add_resource_mmap(
        &mut self,
        resource_path: &str,
        image_path: &PathBuf,
    ) -> Result<&mut Self, Error> {
        let (_, image_mime) = get_file_info(image_path)?;

        // Open the file asynchronously
        let file = tokio::fs::File::open(image_path).await?;
        let file_std = file.into_std().await;

        // Create the memory map and add to EPUB in a blocking task
        let epub = &mut self.epub;
        let path = resource_path.to_string();
        let mime = image_mime.to_string();

        let mmap = unsafe { MmapOptions::new().map(&file_std)? };

        // Add resource directly from memory-mapped data
        epub.add_resource(&path, Cursor::new(&mmap[..]), &mime)?;

        Ok(self)
    }
}

#[async_trait]
impl Generator for EPub {
    fn new(output_path: &str, filename: &str) -> EResult<Self> {
        let mut epub = EpubBuilder::new(ZipLibrary::new()?)?;

        epub.epub_version(EpubVersion::V30);
        epub.stylesheet(include_bytes!("../../templates/template.css").as_slice())?;

        Ok(EPub {
            epub,
            output_path: output_path.to_string(),
            filename: filename.to_string(),
            reading_direction: None,
        })
    }

    async fn add_page(&mut self, image_path: &PathBuf) -> EResult<&mut Self> {
        // In the real implementation we add chapters with multiple images
        // For interface consistency, we'll treat each page as its own chapter
        // The actual implementation would collect pages and add them in chapters

        let (image_extension, _) = get_file_info(&image_path)?;

        // Use the page index as chapter count for this simplified version
        let chapter_count = 1; // In reality, you'd track this
        let i = 0; // Page index within chapter

        let image_name = format!("images/{}/{}.{}", chapter_count, i + 1, image_extension);
        let image_xhtml = generate_xhtml(&image_name)?;

        self.add_resource_mmap(&image_name, image_path).await?;

        self.epub.add_content(EpubContent::new(
            format!("{}-{}.xhtml", chapter_count, i + 1),
            image_xhtml.as_bytes(),
        ))?;

        Ok(self)
    }

    async fn set_metadata(&mut self, title: &str, volume: usize) -> EResult<&mut Self> {
        self.epub
            .metadata("title", &format!("{} | {}", title, volume))?;

        // If the reading direction is set, include it in metadata
        if let Some(direction) = &self.reading_direction {
            match direction {
                Direction::Ltr => self.epub.metadata("direction", "ltr")?,
                Direction::Rtl => self.epub.metadata("direction", "rtl")?,
            };
        }

        self.filename = title.to_string();
        Ok(self)
    }

    async fn save(mut self) -> EResult<()> {
        let output_path = Path::new(&self.output_path);
        let output_file_path = output_path.join(format!("{}.epub", self.filename));
        let file = File::create(&output_file_path)?;
        
        self.epub.generate(file)?;
        Ok(())
    }
}
