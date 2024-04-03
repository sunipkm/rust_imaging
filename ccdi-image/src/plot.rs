use ccdi_common::to_string;
use image::DynamicImage;
use std::{cmp::{min, max}, io::Cursor};
use plotters::{prelude::*, backend::RGBPixel};
use log::{debug, info};

use crate::ImageStats;

// ============================================ PUBLIC =============================================

pub fn render_histogram_as_bmp(stats: &ImageStats, height: usize) -> Result<Vec<u8>, String> {
    info!("render_histogram_as_bmp()");
    let (width, height) = (stats.r.bins.len(), height);
    let mut buffer = vec![0; width*height*3];
    render_plot_to_buffer(&mut buffer, width, height, stats)?;
    save_buffer_as_bmp(&buffer, width, height)
}

// =========================================== PRIVATE =============================================

fn render_plot_to_buffer(
    buffer: &mut [u8],
    width: usize,
    height: usize,
    stats: &ImageStats
) -> Result<(), String> {
    let min_x = min(min(stats.r.min, stats.g.min), stats.b.min) as f32;
    let max_x = max(max(stats.r.max, stats.g.max), stats.b.max) as f32;
    let max_y = max(max(stats.r.max_count(), stats.g.max_count()), stats.b.max_count()) as f32;

    let area = BitMapBackend::<RGBPixel>::with_buffer(
        buffer, (width as u32, height as u32)
    ).into_drawing_area();

    area.fill(&BLACK).unwrap();

    let mut chart = ChartBuilder::on(&area)
        .build_cartesian_2d(min_x..max_x, 0.0..max_y)
        .map_err(to_string)?;

    chart.draw_series(
        LineSeries::new(histogram_to_points(&stats.b), &RGBColor(0, 100, 255))
    ).map_err(to_string)?;

    chart.draw_series(
        LineSeries::new(histogram_to_points(&stats.g), &GREEN)
    ).map_err(to_string)?;

    chart.draw_series(
        LineSeries::new(histogram_to_points(&stats.r), &RED)
    ).map_err(to_string)?;

    Ok(())
}

fn save_buffer_as_bmp(buffer: &[u8], width: usize, height: usize) -> Result<Vec<u8>, String> {
    let mut dynamic = DynamicImage::new_rgb8(width as u32, height as u32);
    let mut offset = 0;

    if let Some(ref mut image) = dynamic.as_mut_rgb8() {
        for (_x, _y, pixel) in image.enumerate_pixels_mut() {
            *pixel = image::Rgb([buffer[offset], buffer[offset + 1], buffer[offset + 2]]);
            offset += 3;
        }
    }

    let mut cursor = Cursor::new(Vec::<u8>::new());

    match dynamic.write_to(&mut cursor, image::ImageOutputFormat::Bmp) {
        Ok(_) => Ok(cursor.into_inner()),
        Err(err) => Err(format!("{:?}", err))
    }
}

fn histogram_to_points(hist: &crate::Histogram) -> Vec<(f32, f32)> {
    let offset = hist.min as f32;
    let multiplier = max(1, hist.max - hist.min) as f32/max(1, hist.bins.len()) as f32;

    hist.bins.iter()
        .enumerate()
        .map(|(index, count)| (index as f32*multiplier + offset, *count as f32))
        .collect()
}