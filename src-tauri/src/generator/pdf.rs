extern crate printpdf;

use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use image_crate::codecs::*;
use printpdf::*;

use crate::prelude::*;
// Needed to override the Error impl from printpdf
use crate::prelude::Error;

const DPI: f32 = 300.0;

pub struct Pdf {
    document: PdfDocumentReference,
}

fn get_image(image_path: &PathBuf) -> Result<Image, Error> {
    let (image_extension, _) = get_file_info(image_path)?;
    let mut image_file = File::open(image_path)?;

    let image = match image_extension {
        "png" => Image::try_from(png::PngDecoder::new(&mut image_file)?)?,
        "jpg" | "jpeg" => Image::try_from(jpeg::JpegDecoder::new(&mut image_file)?)?,
        "bmp" => Image::try_from(bmp::BmpDecoder::new(&mut image_file)?)?,
        _ => return Err(Error::Unsupported("Image format not supported".to_string())),
    };

    Ok(image)
}

fn pixel_to_mm(pixels: usize) -> f32 {
    // https://pixelcalculator.com/de
    // *10 to convert from cm to mm
    (pixels as f32 / DPI) * 2.54 * 10f32
}

impl Pdf {
    pub fn new(title: &str, cover_path: &PathBuf) -> Result<Self, Error> {
        // First get the cover image for the first pages dimensions
        let image = get_image(cover_path)?;

        let (document, page, layer) = PdfDocument::new(
            title,
            Mm(pixel_to_mm(image.image.width.0)),
            Mm(pixel_to_mm(image.image.height.0)),
            "Layer 1",
        );

        let edit_layer = document.get_page(page).get_layer(layer);
        image.add_to_layer(edit_layer, ImageTransform::default());

        Ok(Pdf { document })
    }

    pub fn add_page(&mut self, image_path: &PathBuf) -> Result<&mut Self, Error> {
        let image = get_image(image_path)?;

        let (page, layer) = self.document.add_page(
            Mm(pixel_to_mm(image.image.width.0)),
            Mm(pixel_to_mm(image.image.height.0)),
            "Layer 1",
        );

        let edit_layer = self.document.get_page(page).get_layer(layer);

        image.add_to_layer(edit_layer, ImageTransform::default());

        Ok(self)
    }

    pub fn save(self, output_path: &str, filename: &str) -> Result<(), Error> {
        let mut buffer = BufWriter::new(File::create(format!("{}/{}.pdf", output_path, filename))?);

        self.document.save(&mut buffer)?;
        Ok(())
    }
}
