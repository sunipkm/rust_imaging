use log::info;
use nanocv::{ImgSize, ImgBuf, ImgMut};

use crate::{RawImage, RgbImage};

use super::{lookup::{Offset, LookupTable, scale_lookup_table}, grid::draw_thirds_grid};

// ============================================ PUBLIC =============================================

pub fn debayer_scale_fast(
    input: &RawImage, size: ImgSize
) -> RgbImage<u16> {
    let offsets = &OFFSET_GRBG;
    let r = resize_channel(input, size, offsets.r);
    let g = resize_channel(input, size, offsets.g1);
    let b = resize_channel(input, size, offsets.b);
    let mut image = RgbImage::from(r, g, b).expect("Logic error");

    image
}

// =========================================== PRIVATE =============================================

fn resize_channel(
    image: &RawImage,
    output_size: ImgSize,
    offset: Offset,
) -> ImgBuf<u16> {
    let input_size = ImgSize::new(image.params.area.width, image.params.area.height);
    let lookup = scale_lookup_table(input_size, output_size, offset);
    info!("input_size {:?}, output_size {:?}, offset {:?}", input_size, output_size, offset);
    scale_with_lookup_table(image, &lookup)
}

fn scale_with_lookup_table(image: &RawImage, table: &LookupTable) -> ImgBuf<u16> {
    let w = image.params.area.width;
    let size = ImgSize::new(table.x.len(), table.y.len());
    let mut result = ImgBuf::<u16>::new_init(size, Default::default());

    for line in 0..size.y {
        let dst = result.line_mut(line);
        let input_line = &table.y[line];
        let img = image.data.as_u16().unwrap();
        let img = img.clone().into_vec();
        let src = &img[input_line*w .. (input_line + 1)*w];

        for x in 0..size.x {
            dst[x] = src[table.x[x]];
        }
    }

    result
}

#[derive(Clone, Copy, PartialEq)]
struct ChannelOffsets {
    r: Offset,
    g1: Offset,
    g2: Offset,
    b: Offset
}

const OFFSET_GRBG: ChannelOffsets = ChannelOffsets {
    r: Offset { x: 1, y: 0 },
    g1: Offset { x: 1, y: 1 },
    g2: Offset { x: 0, y: 0 },
    b: Offset { x: 0, y: 1 },
};


