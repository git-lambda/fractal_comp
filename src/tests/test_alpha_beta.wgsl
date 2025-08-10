@group(0) @binding(0) var<storage, read> domain: array<f32>;
@group(0) @binding(1) var<storage, read> range: array<f32>;
@group(0) @binding(2) var<storage, read_write> result: array<f32>;

fn compute_alpha_beta(domain: array<f32, 64>, range: array<f32, 64>, count: u32) -> vec2<f32> {
    var sum_r = 0.0;
    var sum_d = 0.0;
    var sum_dr = 0.0;
    var sum_d2 = 0.0;
    for (var i = 0u; i < count; i++) {
        let r = range[i];
        let d = domain[i];
        sum_r += r;
        sum_d += d;
        sum_dr += d * r;
        sum_d2 += d * d;
    }
    let n = f32(count);
    let denom = n * sum_d2 - sum_d * sum_d;
    let numer = n * sum_dr - sum_d * sum_r;
    let alpha = select(0.0, numer / denom, abs(denom) > 1e-6);
    let beta = (sum_r - alpha * sum_d) / n;
    return vec2<f32>(alpha, beta);
}

@compute @workgroup_size(1)
fn main() {
    var d: array<f32, 64>;
    var r: array<f32, 64>;
    for (var i = 0u; i < 4u; i++) {
        d[i] = domain[i];
        r[i] = range[i];
    }
    let ab = compute_alpha_beta(d, r, 4u);
    result[0] = ab.x;
    result[1] = ab.y;
}
