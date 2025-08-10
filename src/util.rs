use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use crate::encode::EncodedBlock;

pub fn save_fic_file_as_txt(
    path: &Path,
    encoded_blocks: &[EncodedBlock],
    width: u16,
    height: u16,
    block_size: u8,
    stride: u8,
) {
    let file = File::create(path).expect("Failed to create output file");
    let mut writer = BufWriter::new(file);

    // --- Write header ---
    writeln!(writer, "Fractal Encode Debug Dump").unwrap();
    writeln!(writer, "Width: {}", width).unwrap();
    writeln!(writer, "Height: {}", height).unwrap();
    writeln!(writer, "Block size: {}", block_size).unwrap();
    writeln!(writer, "Stride: {}", stride).unwrap();
    writeln!(writer, "Block count: {}", encoded_blocks.len()).unwrap();
    writeln!(writer, "").unwrap();

    // --- Write each encoded block ---
    for (i, block) in encoded_blocks.iter().enumerate() {
        writeln!(
            writer,
            "Block {:03} -> Domain ID: {:03}, Transform: {}, α: {:.3}, β: {:.3}",
            i, block.domain_index(), block.transform_id(), block.alpha, block.beta
        ).unwrap();
    }

    writer.flush().unwrap();
    println!("Saved debug info to '{}'", path.display());
}


pub fn save_fic_file(path: &Path, blocks: &[EncodedBlock], width: u16, height: u16, block_size: u8, stride: u8) {
    use std::io::Write;
    let mut file = std::fs::File::create(path).expect("Failed to create file");

    file.write_all(&width.to_le_bytes()).unwrap();          // 2 bytes
    file.write_all(&height.to_le_bytes()).unwrap();         // 2 bytes
    file.write_all(&[block_size]).unwrap();                 // 1 byte
    file.write_all(&[stride]).unwrap();                     // 1 byte
    file.write_all(&(blocks.len() as u32).to_le_bytes()).unwrap(); // 4 bytes

    for block in blocks {
        file.write_all(&block.meta.to_le_bytes()).unwrap();         // 4 bytes
        file.write_all(&block._unused.to_le_bytes()).unwrap();      // 4 bytes
        file.write_all(&block.alpha.to_bits().to_le_bytes()).unwrap(); // 4 bytes
        file.write_all(&block.beta.to_bits().to_le_bytes()).unwrap();  // 4 bytes
    }

    println!("save_fic_file() writing {} blocks ({} bytes)", blocks.len(), 10 + blocks.len() * 16);

}




pub fn save_debug_txt(path: &Path, encoded_blocks: &[EncodedBlock]) {
    let mut file = std::fs::File::create(path).expect("Failed to create debug txt");
    use std::io::Write;

    writeln!(file, "GPU Output Debug Dump").unwrap();
    writeln!(file, "Total blocks: {}", encoded_blocks.len()).unwrap();

    for (i, b) in encoded_blocks.iter().enumerate() {
        let domain_idx = b.meta & 0xFFFF;
        let transform_id = (b.meta >> 16) & 0xFF;

        writeln!(
            file,
            "Block {:03} -> Domain ID: {:03}, Transform: {}, α: {:.3}, β: {:.3}",
            i, domain_idx, transform_id, b.alpha, b.beta
        ).unwrap();
    }

    println!("Wrote debug dump to {}", path.display());
}
