struct Params {
    tid: u32,
    _pad: u32,
};

@group(0) @binding(0) var<storage, read> input: array<f32, 64>;
@group(0) @binding(1) var<storage, read_write> output: array<f32, 64>;
@group(0) @binding(2) var<uniform> params: Params;

fn xy_to_index(x: u32, y: u32) -> u32 {
    return y * 8u + x;
}

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let tid = params.tid;
// TODO!
}
