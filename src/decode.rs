use crate::encode::EncodedBlock;
use crate::transform::apply_d4_transform;
use image::{GrayImage, Luma};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
pub fn call_test(fic_path: &Path, output_path: &Path, iterations: usize) {
    println!("Entered decode_image()");
    std::thread::sleep(std::time::Duration::from_secs(1)); // force delay to make sure it runs

    let file = match File::open(fic_path) {
        Ok(f) => f,
        Err(e) => {
            println!("❌ Failed to open .fic: {}", e);
            return;
        }
    };

    println!("Opened .fic file successfully");
}

pub fn decode_image(fic_path: &Path, output_path: &Path, iterations: usize) {
    println!("Opening .fic file: {}", fic_path.display());

    let file = File::open(fic_path).expect("Failed to open .fic file");
    let mut reader = BufReader::new(file);

    let mut buf2 = [0u8; 2];
    let mut buf1 = [0u8; 1];
    let mut buf4 = [0u8; 4];

    reader.read_exact(&mut buf2).unwrap();
    let width = u16::from_le_bytes(buf2) as usize;

    reader.read_exact(&mut buf2).unwrap();
    let height = u16::from_le_bytes(buf2) as usize;

    reader.read_exact(&mut buf1).unwrap();
    let block_size = buf1[0] as usize;

    reader.read_exact(&mut buf1).unwrap();
    let stride = buf1[0] as usize;

    reader.read_exact(&mut buf4).unwrap();
    let num_blocks = u32::from_le_bytes(buf4) as usize;

    println!("Header:");
    println!("-> width: {width}, height: {height}");
    println!("-> block size: {block_size}, blocks: {num_blocks}");

    let expected_data_bytes = num_blocks * 16;
    let actual_size = std::fs::metadata(fic_path).unwrap().len() as usize;
    let actual_data_bytes = actual_size - 10;

    println!("-> expected block data: {expected_data_bytes} bytes");
    println!("-> actual   block data: {actual_data_bytes} bytes");

    assert_eq!(
        expected_data_bytes, actual_data_bytes,
        "ERROR: Mismatch in header vs actual .fic block data size"
    );

    // --- Read block data ---
    let mut blocks = Vec::with_capacity(num_blocks);
    for i in 0..num_blocks {
        let mut read_field = |buf: &mut [u8; 4]| {
            reader.read_exact(buf).unwrap();
            *buf
        };
        let meta = u32::from_le_bytes(read_field(&mut buf4));
        let _unused = u32::from_le_bytes(read_field(&mut buf4));
        let alpha = f32::from_le_bytes(read_field(&mut buf4));
        let beta = f32::from_le_bytes(read_field(&mut buf4));

        blocks.push(EncodedBlock {
            meta,
            _unused,
            alpha,
            beta,
        });
    }

    // --- Decode image ---
    let mut current = vec![128.0; width * height];
    let mut new_image = vec![0.0; width * height];
    let bs = block_size;

    for iter in 0..iterations {
        println!("Iteration {iter}...");
        for (i, block) in blocks.iter().enumerate() {
            let blocks_per_row = (width - bs) / stride + 1;

            let bx = (i % blocks_per_row) * stride; // column
            let by = (i / blocks_per_row) * stride; // row

            if bx + bs > width || by + bs > height {
                // Silently skip out-of-bounds blocks
                continue;
            }

            let dom_idx = block.domain_index();
            let transform_id = block.transform_id();

            let dx = dom_idx % (width - bs + 1);
            let dy = dom_idx / (width - bs + 1);

            let mut domain_block = Vec::with_capacity(bs * bs);
            for y in 0..bs {
                for x in 0..bs {
                    let idx = (dy + y) * width + (dx + x);
                    domain_block.push(current[idx]);
                }
            }

            let transformed = apply_d4_transform(&domain_block, bs, transform_id);
            let reconstructed: Vec<f32> = transformed
                .iter()
                .map(|v| block.alpha * v + block.beta)
                .collect();

            for y in 0..bs {
                for x in 0..bs {
                    let dst_idx = (by + y) * width + (bx + x);
                    assert!(
                        dst_idx < new_image.len(),
                        "ERROR: dst_idx {} out of bounds (len = {})",
                        dst_idx,
                        new_image.len()
                    );
                    new_image[dst_idx] = reconstructed[y * bs + x].clamp(0.0, 255.0);
                }
            }
        }

        current.copy_from_slice(&new_image);
    }

    // --- Save output ---
    let mut output = GrayImage::new(width as u32, height as u32);
    for y in 0..height {
        for x in 0..width {
            let val = current[y * width + x].clamp(0.0, 255.0) as u8;
            output.put_pixel(x as u32, y as u32, Luma([val]));
        }
    }

    output
        .save(output_path)
        .expect("Failed to save decoded image");
    println!("Decoded image saved to {}", output_path.display());
}
