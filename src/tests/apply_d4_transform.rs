use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct TransformParams {
    tid: u32,
    _pad: u32,
}
fn init_wgpu() -> (wgpu::Device, wgpu::Queue) {
    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect("no adapter");
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
        .expect("no device")
}
fn main() {
    let (device, queue) = init_wgpu();

    let test_cases = vec![
        (2, "src/tests/transforms2x2.wgsl"),
        (4, "src/tests/transforms4x4.wgsl"),
        (8, "src/tests/transforms8x8.wgsl"),
    ];

    for (block_dim, shader_path) in test_cases {
        println!("\n===============================");
        println!("Testing {}x{} transforms", block_dim, block_dim);

        let block_len = block_dim * block_dim;
        let buffer_bytes = (block_len * std::mem::size_of::<f32>()) as u64;

        let block: Vec<f32> = (1..=block_len).map(|x| x as f32).collect();

        let shader_source = std::fs::read_to_string(shader_path).expect("Failed to read WGSL");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("transform shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let input_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("input block"),
            contents: bytemuck::cast_slice(&block),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let output_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("output block"),
            size: buffer_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniform buffer"),
            size: 8,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(wgpu::BufferSize::new(buffer_bytes).unwrap()),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: Some(wgpu::BufferSize::new(buffer_bytes).unwrap()),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(wgpu::BufferSize::new(8).unwrap()),
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind group"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: input_buf.as_entire_binding(),
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
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            cache: None,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        });

        for tid in 0..8 {
            let params = [tid, 0];
            queue.write_buffer(&uniform_buf, 0, bytemuck::cast_slice(&params));

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("compute pass"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.dispatch_workgroups(1, 1, 1);
            }

            let staging = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("staging"),
                size: buffer_bytes,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            encoder.copy_buffer_to_buffer(&output_buf, 0, &staging, 0, buffer_bytes);
            queue.submit(Some(encoder.finish()));

            let slice = staging.slice(..);
            slice.map_async(wgpu::MapMode::Read, |_| {});
            device.poll(wgpu::PollType::Wait);
            let data = slice.get_mapped_range();
            let result: &[f32] = bytemuck::cast_slice(&data);

            let name = match tid {
                0 => "identity",
                1 => "rot90",
                2 => "rot180",
                3 => "rot270",
                4 => "flipx",
                5 => "flipy",
                6 => "diagonal",
                7 => "anti-diagonal",
                _ => "unknown",
            };

            println!(
                "\n[transform tid={}]{} block {}x{}:",
                tid, name, block_dim, block_dim
            );
            for y in 0..block_dim {
                for x in 0..block_dim {
                    print!("{:>5.1} ", result[y * block_dim + x]);
                }
                println!();
            }
        }
    }
}
