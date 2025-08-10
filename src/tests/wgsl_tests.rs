use wgpu::util::DeviceExt;

fn init_wgpu() -> (wgpu::Device, wgpu::Queue) {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
        .expect("No compatible adapter found");

    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).expect("Failed to create device")
}

fn main() {
    test_alpha_beta();
}

fn test_alpha_beta() {
    let (device, queue) = init_wgpu();

    let shader_source = std::fs::read_to_string("src/tests/test_alpha_beta.wgsl")
        .expect("Failed to read shader");

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("test shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    let domain = vec![1.0f32, 2.0, 3.0, 4.0];
    let range = vec![2.0f32, 4.0, 6.0, 8.0];

    let domain_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("domain"),
        contents: bytemuck::cast_slice(&domain),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let range_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("range"),
        contents: bytemuck::cast_slice(&range),
        usage: wgpu::BufferUsages::STORAGE,
    });

    let output_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("output"),
        size: 8,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
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
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("bind_group"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: domain_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: range_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: output_buf.as_entire_binding() },
        ],
    });

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
        compilation_options: wgpu::PipelineCompilationOptions::default()
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("encoder"),
    });

    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("cpass"),
            timestamp_writes: None,
        });
        cpass.set_pipeline(&pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.dispatch_workgroups(1, 1, 1);
    }

    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("staging"),
        size: 8,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    encoder.copy_buffer_to_buffer(&output_buf, 0, &staging, 0, 8);
    queue.submit(Some(encoder.finish()));

    let slice = staging.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::Wait);

    let data = slice.get_mapped_range();
    let alpha = f32::from_le_bytes(data[0..4].try_into().unwrap());
    let beta = f32::from_le_bytes(data[4..8].try_into().unwrap());

    println!("[test] Got alpha = {:.3}, beta = {:.3}", alpha, beta);
    assert!((alpha - 2.0).abs() < 0.01, "alpha should be ~2.0");
    assert!(beta.abs() < 0.01, "beta should be ~0.0");
}
