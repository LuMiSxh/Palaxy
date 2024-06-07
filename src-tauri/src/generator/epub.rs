use std::fs::File;
use std::path::{Path, PathBuf};

use epub_builder::{EpubBuilder, EpubContent, EpubVersion, ZipLibrary};

use crate::prelude::*;

fn generate_xhtml(image_source: &str) -> Result<String, Error> {
    const TEMPLATE: &str = include_str!("../../templates/template.xhtml");
    let xhtml = TEMPLATE
        .replace("%title%", image_source)
        .replace("%src%", image_source)
        .replace("%alt%", image_source);
    Ok(xhtml)
}

pub struct EPub {
    epub: EpubBuilder<ZipLibrary>,
}

impl EPub {
    pub fn new() -> Result<Self, Error> {
        let mut epub = EpubBuilder::new(ZipLibrary::new()?)?;

        epub.epub_version(EpubVersion::V30);
        epub.stylesheet(include_bytes!("../../templates/template.css").as_slice())?;

        Ok(EPub { epub })
    }

    pub fn set_metadata(&mut self, key: &str, value: &str) -> Result<&mut Self, Error> {
        self.epub.metadata(key, value)?;
        Ok(self)
    }

    pub fn set_cover(&mut self, cover_image_path: &PathBuf) -> Result<&mut Self, Error> {
        let (cover_extension, cover_mime) = get_file_info(cover_image_path)?;
        let cover_file = File::open(cover_image_path)?;

        self.epub.add_cover_image(
            format!("data/cover.{}", cover_extension),
            cover_file,
            cover_mime,
        )?;
        Ok(self)
    }

    pub fn set_lang(&mut self, lang: &str) -> Result<&mut Self, Error> {
        self.epub.set_lang(lang);
        Ok(self)
    }

    pub async fn add_chapter(
        &mut self,
        chapter_count: usize,
        image_paths: &Vec<PathBuf>,
    ) -> Result<&mut Self, Error> {
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

    pub async fn save(&mut self, output_path: &str, file_name: &str) -> Result<(), Error> {
        let output_path = Path::new(output_path);
        let output_file_path = output_path.join(format!("{}.epub", file_name));

        let file = File::create(&output_file_path)?;

        self.epub.generate(file)?;
        Ok(())
    }
}
