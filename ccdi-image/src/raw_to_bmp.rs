use std::{io::Cursor, cmp::{min, max}};

use ccdi_common::RgbImage;
use image::DynamicImage;

// ============================================ PUBLIC =============================================

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Transform {
    pub sub: i32,
    pub gain: i32,
    pub function: TransformFunction
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TransformFunction {
    Linear,
    Sqrt,
    Log2,
}

pub fn rgb_image_to_bmp(image: &RgbImage<u16>, transform: Transform) -> Result<Vec<u8>, String> {
    let mut dynamic = DynamicImage::new_rgb8(image.width() as u32, image.height() as u32);

    if let Some(ref mut gray) = dynamic.as_mut_rgb8() {
        // TODO: use enumerate_rows_mut
        for (x, y, pixel) in gray.enumerate_pixels_mut() {

            *pixel = image::Rgb([
                to_8bit(transform, image.red().line_ref(y as usize)[x as usize] as i32),
                to_8bit(transform, image.green().line_ref(y as usize)[x as usize] as i32),
                to_8bit(transform, image.blue().line_ref(y as usize)[x as usize] as i32),
            ]);
        }
    } else {
        return Err("Cannot convert to rgb 8 image".to_string());
    }

    save_dynamic_image_to_jpeg(&mut dynamic)
}

// =========================================== PRIVATE =============================================

fn save_dynamic_image_to_jpeg(image: &mut DynamicImage) -> Result<Vec<u8>, String> {
    let mut cursor = Cursor::new(Vec::<u8>::new());

    match image.write_to(&mut cursor, image::ImageOutputFormat::Bmp) {
        Ok(_) => Ok(cursor.into_inner()),
        Err(err) => Err(format!("{:?}", err))
    }
}

fn to_8bit(transform: Transform, input: i32) -> u8 {
    match transform.function {
        TransformFunction::Linear => {
            min(255, (max(0, input - transform.sub)*transform.gain) >> 8) as u8
        },
        TransformFunction::Sqrt => {
            let input = (input - transform.sub) as f32;
            let root = (input.sqrt()*255.0) as i32;
            min(255, (max(0, root)*transform.gain) >> 8) as u8
        },
        TransformFunction::Log2 => {
            let input = (input - transform.sub) as f32;
            let root = (input.log2()*4096.0) as i32 - 15000;
            min(255, (max(0, root)*transform.gain) >> 8) as u8
        },
    }
}