use crate::alpha_beta::{compute_alpha_beta, compute_mse};
use crate::block_extractor::*;
use crate::transform::{self, apply_d4_transform};
use crate::util::*;
use image::{DynamicImage, GenericImageView, GrayImage, ImageBuffer, Luma};
use std::path::Path;
use std::vec;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct EncodedBlock {
    pub meta: u32,      // domain_index in lower 16 bits, transform_id in upper 8
    pub _unused: u32,   // unused for now, can use for extra flags
    pub alpha: f32,
    pub beta: f32,
}

impl EncodedBlock {
    pub fn domain_index(&self) -> usize {
        (self.meta & 0xFFFF) as usize
    }

    pub fn transform_id(&self) -> u8 {
        ((self.meta >> 16) & 0xFF) as u8
    }
}


pub fn encode_block(
    range_block: &[f32],
    domain_blocks: &[Vec<f32>],
    block_size: usize,
) -> EncodedBlock {
    let mut best_mse = f32::MAX;
    let mut best_meta = 0u32;
    let mut best_alpha = 0.0;
    let mut best_beta = 0.0;

    for (domain_idx, domain) in domain_blocks.iter().enumerate() {
        for transform_id in 0..8 {
            let transformed = apply_d4_transform(domain, block_size, transform_id);
            let (alpha, beta) = compute_alpha_beta(&transformed, &range_block);
            let mse = compute_mse(&transformed, &range_block, alpha, beta);
            if mse < best_mse {
                best_mse = mse;
                best_meta = ((transform_id as u32) << 16) | (domain_idx as u32 & 0xFFFF);
                best_alpha = alpha;
                best_beta = beta;
            }
        }
    }

    EncodedBlock {
        meta: best_meta,
        _unused: 0,
        alpha: best_alpha,
        beta: best_beta,
    }
}

pub fn encode_image(img_path: &Path, block_size: usize, stride: usize) {
    println!("Trying to load: {}", img_path.display());
    let img = image::open(img_path).expect("Failed to open image!");
    let gs_image = img.to_luma8();

    let (width, height) = gs_image.dimensions();
    println!(
        "Loaded image: {:?} with width: {} and height: {}",
        img_path, width, height
    );

    let image_data: Vec<f32> = gs_image.pixels().map(|p| p[0] as f32).collect();

    let extractor = BlockExtractor::new(
        image_data,
        width as usize,
        height as usize,
        block_size,
        stride,
    );
    let range_blocks = extractor.extract_range_blocks();

    let domain_blocks = extractor.extract_domain_blocks();

    println!("Extracted {} range blocks!", range_blocks.len());
    println!("Extracted {} domain blocks!", domain_blocks.len());

    let mut encoded_blocks = Vec::new();

    for range in &range_blocks {
        let encoded = encode_block(range, &domain_blocks, block_size);
        encoded_blocks.push(encoded);
    }

    let output_path = Path::new("output.fic");

    save_fic_file(
        output_path,
        &encoded_blocks,
        width as u16,
        height as u16,
        block_size as u8,
        stride as u8,
    );
}
