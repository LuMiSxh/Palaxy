use std::path::PathBuf;

use crate::prelude::*;

const DPI: f32 = 300.0;

pub struct Pdf {
    document: usize,
}

fn pixel_to_mm(pixels: usize) -> f32 {
    // https://pixelcalculator.com/de
    // *10 to convert from cm to mm
    (pixels as f32 / DPI) * 2.54 * 10f32
}

impl Pdf {
    pub fn new(_: &str, _: &PathBuf) -> Result<Self, Error> {
        pixel_to_mm(3);
        let temp = Pdf {
            document: 0,
        };
        
        temp.document;
        unimplemented!("Pdf::new");
    }

    pub fn add_page(&mut self, _: &PathBuf) -> Result<&mut Self, Error> {
        unimplemented!("Pdf::add_page");
    }

    pub fn save(self, _: &str, _: &str) -> Result<(), Error> {
        unimplemented!("Pdf::save");
    }
}
