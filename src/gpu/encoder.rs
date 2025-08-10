use std::path::Path;

use crate::{
    decode,
    encode::EncodedBlock,
    util::{save_debug_txt, save_fic_file_as_txt},
};
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::{
    block_extractor::{self, BlockExtractor},
    encode,
    util::save_fic_file,
};

pub struct EncodeParams {
    pub image_width: u32,
    pub image_height: u32,
    pub range_size: u32,
    pub domain_size: u32,
    pub stride: u32,
}

fn init_wgpu() -> (wgpu::Device, wgpu::Queue) {
    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect("no adapter");
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
        .expect("no device")
}

pub fn process_gpu_encode(img_path: &Path, encode_params: EncodeParams, block_dim: u32) {
    println!("Trying to load: {}", img_path.display());
    let img = image::open(img_path).expect("Failed to open image!");
    let gs_image = img.to_luma8();

    let (width, height) = gs_image.dimensions();
    println!(
        "[gpu] Loaded image: {:?} with width: {} and height: {}",
        img_path, width, height
    );

    let image_data: Vec<f32> = gs_image.pixels().map(|p| p[0] as f32).collect();

    // ---------------- hand over image data, block size, and stride to wgpu compute ----------------

    let encoded_blocks = encode_on_gpu(
        image_data,
        encode_params.image_width,
        encode_params.image_height,
        encode_params.range_size,
        encode_params.domain_size,
        encode_params.stride,
    );

    let output_path = Path::new("output.fic");
    println!("[gpu] Saving encoded image to: {:?}", output_path);
    save_fic_file(
        output_path,
        &encoded_blocks,
        width as u16,
        height as u16,
        encode_params.range_size as u8,
        encode_params.stride as u8,
    );
    println!("[gpu] Saving encoded image debug");
    let debug_path = Path::new("fic_debug.txt");
    save_fic_file_as_txt(
        debug_path,
        &encoded_blocks,
        width as u16,
        height as u16,
        block_dim as u8,
        encode_params.stride as u8,
    );
    let to_decode_path = output_path.with_extension("decoded.png");

    println!("[gpu] Decoding .fic into {:?}", to_decode_path);
    decode::decode_image(output_path.as_ref(), &to_decode_path, 15);
    println!("[gpu] Done decoding.");
}

fn encode_on_gpu(
    image_data: Vec<f32>,
    img_width: u32,
    img_height: u32,
    range_size: u32,
    domain_size: u32,
    stride: u32,
) -> Vec<EncodedBlock> {
    let (device, queue) = init_wgpu();

    let shader_path = match range_size {
        2 => "src/gpu/transform_and_compare2x2.wgsl",
        4 => "src/gpu/transform_and_compare4x4.wgsl",
        8 => "src/gpu/transform_and_compare8x8.wgsl",
        _ => panic!("Unsupported block size"),
    };

    let shader_source = std::fs::read_to_string(shader_path).expect("Failed to load shader");
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("transform_and_compare shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    let image_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("image buffer"),
        contents: bytemuck::cast_slice(&image_data),
        usage: wgpu::BufferUsages::STORAGE,
    });

    // Compute number of range blocks
    let range_blocks_x = (img_width - range_size) / stride + 1;
    let range_blocks_y = (img_height - range_size) / stride + 1;
    let total_ranges = (range_blocks_x * range_blocks_y) as usize;

    let output_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("output encoded blocks"),
        size: (total_ranges * std::mem::size_of::<EncodedBlock>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    // Uniforms (can expand later)
    #[repr(C)]
    #[derive(Clone, Copy, Pod, Zeroable)]
    struct Params {
        img_width: u32,
        img_height: u32,
        range_size: u32,
        domain_size: u32,
        stride: u32,
        range_blocks_x: u32,
        range_blocks_y: u32,
        _pad: u32,
    }

    let params = Params {
        img_width,
        img_height,
        range_size,
        domain_size,
        stride,
        range_blocks_x,
        range_blocks_y,
        _pad: 0,
    };

    let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("uniforms"),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    /*
    Binding 0: Image data
    Binding 1: Output (encoded blocks)
    Binding 2: Encode Parameters
     */

    // Bind group layout and bind group
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: image_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: output_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: uniform_buf.as_entire_binding(),
            },
        ],
        label: Some("bind group"),
    });

    // Pipeline
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("pipeline layout"),
        bind_group_layouts: &[&layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("compute pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("main"),
        cache: None,
        compilation_options: wgpu::PipelineCompilationOptions::default(),
    });

    // Encode + dispatch
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("encoder"),
    });

    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("main pass"),
            timestamp_writes: None,
        });
        cpass.set_pipeline(&pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.dispatch_workgroups(range_blocks_x, range_blocks_y, 1);
    }
    // Staging buffer for results
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("staging"),
        size: (total_ranges * std::mem::size_of::<EncodedBlock>()) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    println!(
        "[gpu] About to map result buffer of size: {}",
        staging.size()
    );

    encoder.copy_buffer_to_buffer(&output_buf, 0, &staging, 0, staging.size());
    queue.submit(Some(encoder.finish()));

    // Wait + map
    let slice = staging.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::Wait);

    let data = slice.get_mapped_range();
    let vec: Vec<EncodedBlock> = bytemuck::cast_slice(&data).to_vec(); // ✅ clone
    drop(data); // ✅ unmap range safely
    staging.unmap(); // ✅ avoid lingering mapping

    vec // ✅ return result cleanly
}
