use std::path::Path;
use std::time;
use std::{fs, io};
mod alpha_beta;
mod block_extractor;
mod decode;
mod encode;
mod gpu;
mod transform;
mod util;

fn main() {
    println!("[main] Starting program...");

    println!("[main] Enter the block size:");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let block_size: usize = input.trim().parse().expect("Invalid block size");
    println!("[main] Using block size: {}", block_size);

    println!("[main] Enter the stride:");
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    let stride: usize = input.trim().parse().expect("Invalid stride");
    println!("[main] Using stride: {}", stride);

    let default_img_path = String::from("test_imgs/lena256.png");
    println!("[main] Mode selection...");
    println!("1. Single image (default: {default_img_path})");
    println!("2. Batch process");

    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    let mode = input.trim();

    if mode != "1" {
        println!("[main] Batch mode not implemented");
        return;
    }

    println!("[main] Single image selected.");
    println!(
        "Enter image path or press enter to use default: {}",
        default_img_path
    );
    print!("> ");
    io::Write::flush(&mut io::stdout()).unwrap();

    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    let trimmed = input.trim().to_string();

    let path_str = if trimmed.is_empty() {
        default_img_path.as_str()
    } else {
        trimmed.as_str()
    };
    let to_encode_path = Path::new(path_str);
    let fic_path = Path::new("output.fic");

    println!("[main] Choose encoding method:");
    println!("1. CPU");
    println!("2. GPU");

    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    let method = input.trim();

    if method == "1" {
        println!("[main] CPU encoding selected.");
        encode::encode_image(to_encode_path, block_size, stride);
        println!("[main] Preparing decode step...");
        let output_path = to_encode_path.with_extension("decoded.png");
        println!(" → Source fic: {}", fic_path.display());
        println!(" → Output path: {}", output_path.display());

        assert!(fic_path.exists(), "ERROR: .fic file not found");
        assert!(
            output_path.to_str().is_some(),
            "ERROR: output_path is not valid UTF-8"
        );

        println!("[main] Calling decode_image...");
        decode::decode_image(fic_path, &output_path, 10);
        println!("[main] Finished decode.");
    } else if method == "2" {
        println!("[main] GPU encoding selected.");

        println!("Trying to load: {}", to_encode_path.display());
        let img = image::open(to_encode_path).expect("Failed to open image!");
        let gs_image = img.to_luma8();

        let (width, height) = gs_image.dimensions();
        println!(
            "Loaded image: {:?} with width: {} and height: {}",
            to_encode_path, width, height
        );

        println!("[main] Entering GPU block...");
        {
            println!("[gpu] Initializing encoder...");
            let encode_params = gpu::encoder::EncodeParams {
                image_width: width,
                image_height: height,
                range_size: block_size as u32,
                domain_size: block_size as u32,
                stride: stride as u32,
            };
            let gpu_encode_start = time::Instant::now();
            gpu::encoder::process_gpu_encode(to_encode_path, encode_params, block_size as u32);

            println!("[gpu] Finished Processing image...");
            println!(
                "Total processing time for this image: {:?}",
                gpu_encode_start.elapsed()
            );
        }
    } else {
        println!("Invalid method selected.");
        return;
    }
}
