use nanocv::{ImgSize, ImgBuf, ImgMut};

use crate::{RawImage, RgbImage, RenderingType};
use serialimage::DynamicImage;

use super::{lookup::{Offset, LookupTable, scale_lookup_table}, grid::draw_thirds_grid};

// ============================================ PUBLIC =============================================

pub fn debayer_scale_fast(
    input: &RawImage, size: ImgSize, rendering: RenderingType
) -> RgbImage<u16> {
    let offsets = &OFFSET_GRBG;
    let r = resize_channel(input, size, offsets.r, rendering);
    let g = resize_channel(input, size, offsets.g1, rendering);
    let b = resize_channel(input, size, offsets.b, rendering);
    let mut image = RgbImage::from(r, g, b).expect("Logic error");

    if rendering == RenderingType::Corners1x {
        draw_thirds_grid(&mut image);
    }

    image
}

// =========================================== PRIVATE =============================================

fn resize_channel(
    image: &RawImage,
    output_size: ImgSize,
    offset: Offset,
    rendering: RenderingType
) -> ImgBuf<u16> {
    let input_size = ImgSize::new(image.params.area.width, image.params.area.height);
    let lookup = scale_lookup_table(input_size, output_size, offset, rendering);
    scale_with_lookup_table(image, &lookup)
}

fn scale_with_lookup_table(image: &RawImage, table: &LookupTable) -> ImgBuf<u16> {
    let w = image.params.area.width;
    let size = ImgSize::new(table.x.len(), table.y.len());
    let mut result = ImgBuf::<u16>::new_init(size, Default::default());

    for line in 0..size.y {
        let dst = result.line_mut(line);
        let input_line = &table.y[line];
        let img: DynamicImage = DynamicImage::from(image.data.clone());
        let img = img.as_luma16().unwrap().clone().into_vec();
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


