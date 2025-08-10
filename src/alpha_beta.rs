pub fn compute_alpha_beta(domain: &[f32], range: &[f32]) -> (f32, f32) {
    let mut sum_r = 0.0;
    let mut sum_d = 0.0;
    let mut sum_dr = 0.0;
    let mut sum_d2 = 0.0;

    for i in (0..domain.len()) {
        let d = domain[i];
        let r = range[i];

        sum_r += r;
        sum_d += d;
        sum_dr += d * r;
        sum_d2 += d * d;
    }

    let n = domain.len() as f32;
    let numerator = n * sum_dr - sum_d * sum_r;
    let denominator = n * sum_d2 - sum_d * sum_d;
    let alpha = if denominator.abs() < 1e-6 {
        0.0
    } else {
        numerator / denominator
    };
    let beta = (sum_r - alpha * sum_d) / n;

    (alpha, beta)
}

pub fn compute_mse(domain: &[f32], range: &[f32], alpha: f32, beta: f32) -> f32 {
    let mut mse = 0.0;
    let n = domain.len() as f32;

    for i in 0..domain.len() {
        let d = alpha * domain[i] + beta;
        let r = range[i];
        let error = r - d;
        mse += error * error;
    }

    mse /= n;

    return mse;
}
