use std::fs::{read, File};
use std::io::Write;
use std::path::PathBuf;

use zip::write::{SimpleFileOptions};
use zip::{CompressionMethod, ZipWriter};

use crate::prelude::*;

pub struct Cbz {
    zip: ZipWriter<File>,
    options: SimpleFileOptions,
    page_index: usize,
}

impl Cbz {
    pub fn new(output_path: &str, filename: &str) -> Result<Self, Error> {
        let options: SimpleFileOptions = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o755);

        let file = File::create(format!("{}/{}.cbz", output_path, filename))?;
        let zip = ZipWriter::new(file);

        Ok(Cbz {
            zip,
            options,
            page_index: 0,
        })
    }

    pub fn add_page(&mut self, image_path: &PathBuf) -> Result<&mut Self, Error> {
        let (image_extension, _) = get_file_info(image_path)?;
        let mut image_file = read(image_path)?;

        self.page_index += 1;

        self.zip.start_file(
            format!("page_{:03}.{}", self.page_index, image_extension),
            self.options,
        )?;
        self.zip.write_all(&mut image_file)?;

        Ok(self)
    }

    pub fn set_comicinfo(&mut self, title: &str, volume: usize) -> Result<&mut Self, Error> {
        const TEMPLATE: &str = include_str!("../../templates/template.xml");

        let xml = TEMPLATE
            .replace("%title%", title)
            .replace("%volume%", &*volume.to_string())
            .replace("%pagecount%", &*self.page_index.to_string());

        self.zip.start_file("ComicInfo.xml", self.options)?;
        self.zip.write_all(xml.as_bytes())?;

        Ok(self)
    }

    pub fn save(self) -> Result<(), Error> {
        self.zip.finish()?;
        Ok(())
    }
}
