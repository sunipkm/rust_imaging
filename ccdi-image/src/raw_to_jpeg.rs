use std::{io::Cursor, cmp::{min, max}};

use ccdi_common::{RgbImage};
use image::{DynamicImage};

// ============================================ PUBLIC =============================================

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Transform {
    pub sub: i32,
    pub gain: i32,
}

pub fn rgb_image_to_jpeg(image: &RgbImage<u16>, transform: Transform) -> Result<Vec<u8>, String> {
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
        return Err(format!("Cannot convert to rgb 8 image"));
    }

    save_dynamic_image_to_jpeg(&mut dynamic)
}

// =========================================== PRIVATE =============================================

fn save_dynamic_image_to_jpeg(image: &mut DynamicImage) -> Result<Vec<u8>, String> {
    let mut cursor = Cursor::new(Vec::<u8>::new());

    match image.write_to(&mut cursor, image::ImageOutputFormat::Jpeg(95)) {
        Ok(_) => Ok(cursor.into_inner()),
        Err(err) => Err(format!("{:?}", err))
    }
}

fn to_8bit(transform: Transform, input: i32) -> u8 {
    min(255, max(0, input - transform.sub)*transform.gain >> 8) as u8
}