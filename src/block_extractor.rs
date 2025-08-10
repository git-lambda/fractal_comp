pub struct BlockExtractor {
    pub image: Vec<f32>,
    pub width: usize,
    pub height: usize,
    pub block_size: usize,
    pub stride: usize,
}

impl BlockExtractor {
    pub fn new(
        image: Vec<f32>,
        width: usize,
        height: usize,
        block_size: usize,
        stride: usize,
    ) -> Self {
        Self {
            image,
            width,
            height,
            block_size,
            stride,
        }
    }

    pub fn extract_blocks(&self, stride: usize) -> Vec<Vec<f32>> {
        let mut blocks = Vec::new();
        let b = self.block_size;

        for y in (0..=self.height - b).step_by(stride) {
            for x in (0..=self.width - b).step_by(stride) {
                let mut block = Vec::with_capacity(b * b);

                for dy in 0..b {
                    for dx in 0..b {
                        let px = x + dx;
                        let py = y + dy;
                        let idx = py * self.width + px;
                        block.push(self.image[idx]);
                    }
                }
                blocks.push(block);
            }
        }
        blocks
    }

    // Since Range Blocks don't overlap, use block size as the stride
    pub fn extract_range_blocks(&self) -> Vec<Vec<f32>> {
        self.extract_blocks(self.block_size)
    }

    // Domain Blocks on the other hand, can and SHOULD overlap in most cases
    // for better accuracy
    pub fn extract_domain_blocks(&self) -> Vec<Vec<f32>> {
        self.extract_blocks(self.stride)
    }

    pub fn downsample(&self, factor: usize) -> Vec<f32> {
        let mut downsampled = Vec::new();
        let step = factor;

        let new_width = self.width / step;
        let new_height = self.height / step;

        for y in (0..new_height) {
            for x in (0..new_width) {
                let mut sum = 0.0;

                for dy in 0..step {
                    for dx in 0..step {
                        let src_x = x * step + dx;
                        let src_y = y * step + dy;
                        let idx = src_y * self.width + src_x;
                        sum += self.image[idx];
                    }
                }
                let avg = sum / (step * step) as f32;
                downsampled.push(avg);
            }
        }
        downsampled
    }
}
