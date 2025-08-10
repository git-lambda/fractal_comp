
struct EncodedBlock {
    meta_data: u32,
    _unused: u32,
    alpha: f32,
    beta: f32,
};

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

@group(0) @binding(0) var<storage, read> image: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<EncodedBlock>;
@group(0) @binding(2) var<uniform> params: Params;


fn load_block_2x2(x: u32, y: u32, stride: u32, width: u32) -> array<f32, 4> {
  return array<f32, 4>(
    image[(y + 0u) * width + (x + 0u)],
    image[(y + 0u) * width + (x + 1u)],
    image[(y + 1u) * width + (x + 0u)],
    image[(y + 1u) * width + (x + 1u)]
  );
}
fn apply_d4_2x2(input: array<f32, 4>, tid: u32) -> array<f32, 4> {

    var temp: array<f32, 4>;
    var out: array<f32, 4>;

    for (var i = 0u; i < 4u; i++) {
        temp[i] = input[i];
    }

    // Index layout:
    // 0 1
    // 2 3

    if (tid == 0u) {
        for (var i = 0u; i < 4u; i++) {
            out[i] = temp[i];
        }
    } else if (tid == 1u) {
        out[0] = temp[2]; out[1] = temp[0];
        out[2] = temp[3]; out[3] = temp[1];
    } else if (tid == 2u) {
        out[0] = temp[3]; out[1] = temp[2];
        out[2] = temp[1]; out[3] = temp[0];
    } else if (tid == 3u) {
        out[0] = temp[1]; out[1] = temp[3];
        out[2] = temp[0]; out[3] = temp[2];
    } else if (tid == 4u) {
        out[0] = temp[1]; out[1] = temp[0];
        out[2] = temp[3]; out[3] = temp[2];
    } else if (tid == 5u) {
        out[0] = temp[2]; out[1] = temp[3];
        out[2] = temp[0]; out[3] = temp[1];
    } else if (tid == 6u) {
        out[0] = temp[0]; out[1] = temp[2];
        out[2] = temp[1]; out[3] = temp[3];
    } else if (tid == 7u) {
        out[0] = temp[3]; out[1] = temp[1];
        out[2] = temp[2]; out[3] = temp[0];
    }

    return out;
}


fn compute_ab_mse(a: array<f32, 4>, b: array<f32, 4>) -> vec3<f32> {
  var sum_a = 0.0;
  var sum_b = 0.0;
  var sum_ab = 0.0;
  var sum_bb = 0.0;

  for (var i = 0u; i < 4u; i++) {
    sum_a += a[i];
    sum_b += b[i];
    sum_ab += a[i] * b[i];
    sum_bb += b[i] * b[i];
  }

  let alpha = (4.0 * sum_ab - sum_a * sum_b) / (4.0 * sum_bb - sum_b * sum_b + 0.0001);
  let beta = (sum_a - alpha * sum_b) / 4.0;

  var mse = 0.0;
  for (var i = 0u; i < 4u; i++) {
    let diff = a[i] - (alpha * b[i] + beta);
    mse += diff * diff;
  }

  return vec3<f32>(alpha, beta, mse / 4.0);
}
@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
  let x = gid.x;
  let y = gid.y;

  if (x >= params.range_blocks_x || y >= params.range_blocks_y) {
    return;
  }

  let range_x = x * params.stride;
  let range_y = y * params.stride;

  let range_block = load_block_2x2(range_x, range_y, 1u, params.img_width);

  var best_mse = 1e10;
  var best_tid = 0u;
  var best_did = 0u;
  var best_ab = vec2<f32>(0.0, 0.0);

  let domain_stride = params.stride;

  var did = 0u;
  for (var dy = 0u; dy + 1u < params.img_height; dy += domain_stride) {
    for (var dx = 0u; dx + 1u < params.img_width; dx += domain_stride) {
      let domain_block = load_block_2x2(dx, dy, 1u, params.img_width);

      for (var tid = 0u; tid < 8u; tid++) {
        let transformed = apply_d4_2x2(domain_block, tid);
        let abmse = compute_ab_mse(range_block, transformed);
        if (abmse.z < best_mse) {
          best_mse = abmse.z;
          best_ab = abmse.xy;
          best_tid = tid;
          best_did = did;

          if (best_mse < 0.0001) {
          break;
        }
        }
      }

    if (best_mse < 0.0001) {
      break;
    }

      did += 1u;
    }
    if (best_mse < 0.0001) {
      break;
    }

  }

  let out_index = y * params.range_blocks_x + x;
output[out_index] = EncodedBlock(
    (best_tid << 16u) | (best_did & 0xFFFFu), // meta_data
    0u,                                       // _unused
    best_ab.x,                                // alpha
    best_ab.y                                 // beta
);


}
