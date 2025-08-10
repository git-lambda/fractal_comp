fn apply_d4_2x2(input: array<f32, 4>, tid: u32) -> array<f32, 4> {

    var temp: array<f32, 4>;
    for (var i = 0u; i < 4u; i++) {
        temp[i] = input[i];
    }

    // Index layout:
    // 0 1
    // 2 3

    if (tid == 0u) {
        for (var i = 0u; i < 4u; i++) {
            output[i] = temp[i];
        }
    } else if (tid == 1u) {
        output[0] = temp[2]; output[1] = temp[0];
        output[2] = temp[3]; output[3] = temp[1];
    } else if (tid == 2u) {
        output[0] = temp[3]; output[1] = temp[2];
        output[2] = temp[1]; output[3] = temp[0];
    } else if (tid == 3u) {
        output[0] = temp[1]; output[1] = temp[3];
        output[2] = temp[0]; output[3] = temp[2];
    } else if (tid == 4u) {
        output[0] = temp[1]; output[1] = temp[0];
        output[2] = temp[3]; output[3] = temp[2];
    } else if (tid == 5u) {
        output[0] = temp[2]; output[1] = temp[3];
        output[2] = temp[0]; output[3] = temp[1];
    } else if (tid == 6u) {
        output[0] = temp[0]; output[1] = temp[2];
        output[2] = temp[1]; output[3] = temp[3];
    } else if (tid == 7u) {
        output[0] = temp[3]; output[1] = temp[1];
        output[2] = temp[2]; output[3] = temp[0];
    }
}