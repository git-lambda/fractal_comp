pub fn apply_d4_transform(block: &[f32], width: usize, transform_id: u8) -> Vec<f32> {
    /*
    0 - No change
    1 - Rotate 90°
    2 - Rotate 180°
    3 - Rotate 270°
    4 - Flip horizontal
    5 - Flip vertical
    6 - Diag flip (y = x)
    7 - Diag flip (y = -x)
    */
    let mut output = vec![0.0; width * width];

    for y in 0..width {
        for x in 0..width {
            let in_idx = y * width + x;
            let (tx, ty) = match transform_id {
                0 => (x, y),
                1 => (width - 1 - y, x),
                2 => (width - 1 - x, width - 1 - y),
                3 => (y, width - 1 - x),
                4 => (width - 1 - x, y),
                5 => (x, width - 1 - y),
                6 => (y, x),
                7 => (width - 1 - y, width - 1 - x),
                _ => panic!("Invalid transform ID: {}", transform_id),
            };

            let out_idx = ty * width + tx;
            output[out_idx] = block[in_idx];
        }
    }

    output
}
