struct Params {
    tid: u32,
    _pad: u32, // align to 8 bytes
};

@group(0) @binding(0) var<storage, read> input: array<f32, 16>;
@group(0) @binding(1) var<storage, read_write> output: array<f32, 16>;
@group(0) @binding(2) var<uniform> params: Params;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let tid = params.tid;

    var temp: array<f32, 16>;

    for (var i = 0u; i < 16u; i++) {
        temp[i] = input[i];
    }

    if (tid == 0u) {
        for (var i = 0u; i < 16u; i++) {
            output[i] = temp[i];
        }
    } else if (tid == 1u) {
        output[ 0] = temp[12]; output[ 1] = temp[ 8]; output[ 2] = temp[ 4]; output[ 3] = temp[ 0];
        output[ 4] = temp[13]; output[ 5] = temp[ 9]; output[ 6] = temp[ 5]; output[ 7] = temp[ 1];
        output[ 8] = temp[14]; output[ 9] = temp[10]; output[10] = temp[ 6]; output[11] = temp[ 2];
        output[12] = temp[15]; output[13] = temp[11]; output[14] = temp[ 7]; output[15] = temp[ 3];
    }else if (tid == 2u) {
        // rotate 180
        output[ 0] = temp[15]; output[ 1] = temp[14]; output[ 2] = temp[13]; output[ 3] = temp[12];
        output[ 4] = temp[11]; output[ 5] = temp[10]; output[ 6] = temp[ 9]; output[ 7] = temp[ 8];
        output[ 8] = temp[ 7]; output[ 9] = temp[ 6]; output[10] = temp[ 5]; output[11] = temp[ 4];
        output[12] = temp[ 3]; output[13] = temp[ 2]; output[14] = temp[ 1]; output[15] = temp[ 0];
    }else if (tid == 3u) {
        // rotate 270
        output[ 0] = temp[ 3]; output[ 1] = temp[ 7]; output[ 2] = temp[11]; output[ 3] = temp[15];
        output[ 4] = temp[ 2]; output[ 5] = temp[ 6]; output[ 6] = temp[10]; output[ 7] = temp[14];
        output[ 8] = temp[ 1]; output[ 9] = temp[ 5]; output[10] = temp[ 9]; output[11] = temp[13];
        output[12] = temp[ 0]; output[13] = temp[ 4]; output[14] = temp[ 8]; output[15] = temp[12];
    }else if (tid == 4u) {
        // flip horizontally (flip x)
        output[ 0] = temp[ 3]; output[ 1] = temp[ 2]; output[ 2] = temp[ 1]; output[ 3] = temp[ 0];
        output[ 4] = temp[ 7]; output[ 5] = temp[ 6]; output[ 6] = temp[ 5]; output[ 7] = temp[ 4];
        output[ 8] = temp[11]; output[ 9] = temp[10]; output[10] = temp[ 9]; output[11] = temp[ 8];
        output[12] = temp[15]; output[13] = temp[14]; output[14] = temp[13]; output[15] = temp[12];
    }else if (tid == 5u) {
        // flip vertically (flip y)
        output[ 0] = temp[12]; output[ 1] = temp[13]; output[ 2] = temp[14]; output[ 3] = temp[15];
        output[ 4] = temp[ 8]; output[ 5] = temp[ 9]; output[ 6] = temp[10]; output[ 7] = temp[11];
        output[ 8] = temp[ 4]; output[ 9] = temp[ 5]; output[10] = temp[ 6]; output[11] = temp[ 7];
        output[12] = temp[ 0]; output[13] = temp[ 1]; output[14] = temp[ 2]; output[15] = temp[ 3];
    }else if (tid == 6u) {
        // diagonal flip (transpose)
        output[ 0] = temp[ 0]; output[ 1] = temp[ 4]; output[ 2] = temp[ 8]; output[ 3] = temp[12];
        output[ 4] = temp[ 1]; output[ 5] = temp[ 5]; output[ 6] = temp[ 9]; output[ 7] = temp[13];
        output[ 8] = temp[ 2]; output[ 9] = temp[ 6]; output[10] = temp[10]; output[11] = temp[14];
        output[12] = temp[ 3]; output[13] = temp[ 7]; output[14] = temp[11]; output[15] = temp[15];
    }else if (tid == 7u) {
        // anti-diagonal flip (transpose + rot180)
        output[ 0] = temp[15]; output[ 1] = temp[11]; output[ 2] = temp[ 7]; output[ 3] = temp[ 3];
        output[ 4] = temp[14]; output[ 5] = temp[10]; output[ 6] = temp[ 6]; output[ 7] = temp[ 2];
        output[ 8] = temp[13]; output[ 9] = temp[ 9]; output[10] = temp[ 5]; output[11] = temp[ 1];
        output[12] = temp[12]; output[13] = temp[ 8]; output[14] = temp[ 4]; output[15] = temp[ 0];
    }
    // Add remaining branches later
}
