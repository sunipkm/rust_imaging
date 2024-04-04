use nanocv::ImgSize;

// ============================================ PUBLIC =============================================

pub struct LookupTable {
    pub x: Vec<usize>,
    pub y: Vec<usize>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Offset {
    pub x: usize,
    pub y: usize,
}

pub fn scale_lookup_table(
    input: ImgSize, output: ImgSize, offset: Offset
) -> LookupTable {
    let half_input = ImgSize { x: input.x/2, y: input.y/2 };

    apply_offsets(
        lookup_table_full_image(half_input, output),
        offset
    )
}

// =========================================== PRIVATE =============================================

fn apply_offsets(table: LookupTable, offset: Offset) -> LookupTable {
    LookupTable {
        x: table.x.into_iter().map(|x| x*2 + offset.x).collect(),
        y: table.y.into_iter().rev().map(|y| y*2 + offset.y).collect(),
    }
}

fn lookup_table_full_image(input: ImgSize, output: ImgSize) -> LookupTable {
    LookupTable {
        x: (0..output.x).map(move |x| x*input.x/output.x).collect(),
        y: (0..output.y).map(move |y| y*input.y/output.y).collect(),
    }
}

fn lookup_table_center(input: ImgSize, output: ImgSize) -> LookupTable {
    let (half_input_x, half_input_y) = (input.x as isize/2, input.y as isize/2);
    let start_x = half_input_x - (output.x/2) as isize - 1;
    let start_y = half_input_y - (output.y/2) as isize - 1;

    LookupTable {
        x: (0..output.x as isize)
            .map(move |x| (start_x + x).clamp(0, input.x as isize - 1) as usize)
            .collect(),
        y: (0..output.y as isize)
            .map(move |y| (start_y + y).clamp(0, input.y as isize - 1) as usize)
            .collect(),
    }
}

fn lookup_table_corners(input: ImgSize, output: ImgSize) -> LookupTable {
    LookupTable {
        x: (0..output.x).map(move |x| apply_corners(x, input.x, output.x)).collect(),
        y: (0..output.y).map(move |y| apply_corners(y, input.y, output.y)).collect(),
    }
}

fn apply_corners(current: usize, input: usize, output: usize) -> usize {
    let third = output as isize/3;

    match current as isize {
        pos if pos < third => pos,
        pos if pos >= 2*third => pos + input as isize - output as isize,
        pos => input as isize/2 - output as isize/2 + pos,
    }.clamp(0,  input as isize - 1) as usize
}

// ============================================= TEST ==============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_full_no_scale() {
        let input = ImgSize::new(8, 4);
        let output = ImgSize::new(4, 2);
        let table = lookup_table_full_image(input, output);
        assert_eq!(table.x, vec![0, 2, 4, 6]);
        assert_eq!(table.y, vec![0, 2]);
    }

    #[test]
    fn test_lookup_full_2x_scale() {
        let input = ImgSize::new(16, 8);
        let output = ImgSize::new(4, 2);
        let table = lookup_table_full_image(input, output);
        assert_eq!(table.x, vec![0, 4, 8, 12]);
        assert_eq!(table.y, vec![0, 4]);
    }

    #[test]
    fn test_lookup_center() {
        let input = ImgSize::new(18, 18);
        let output = ImgSize::new(5, 3);
        let table = lookup_table_center(input, output);
        assert_eq!(table.x, vec![6, 7, 8, 9, 10]);
        assert_eq!(table.y, vec![7, 8, 9]);
    }

    #[test]
    fn test_lookup_center_large() {
        let input = ImgSize::new(180, 90);
        let output = ImgSize::new(5, 3);
        let table = lookup_table_center(input, output);
        assert_eq!(table.x, vec![87, 88, 89, 90, 91]);
        assert_eq!(table.y, vec![43, 44, 45]);
    }

    #[test]
    fn test_lookup_corners() {
        let input = ImgSize::new(18, 18);
        let output = ImgSize::new(6, 3);
        let table = lookup_table_corners(input, output);
        assert_eq!(table.x, vec![0, 1, 8, 9, 16, 17]);
        assert_eq!(table.y, vec![0, 9, 17]);
    }
}