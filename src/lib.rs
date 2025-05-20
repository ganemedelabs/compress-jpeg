use wasm_bindgen::prelude::*;
use js_sys::Uint8ClampedArray;
use wasm_bindgen_futures::future_to_promise;

#[wasm_bindgen]
pub struct ImageData {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

#[wasm_bindgen]
impl ImageData {
    #[wasm_bindgen(constructor)]
    pub fn new(data: Uint8ClampedArray, width: u32, height: u32) -> ImageData {
        let data_vec = data.to_vec();
        ImageData {
            width,
            height,
            data: data_vec,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn data(&self) -> Uint8ClampedArray {
        Uint8ClampedArray::from(self.data.as_slice())
    }
}

#[wasm_bindgen]
pub async fn compress_jpeg(
    image_data: ImageData,
    quality: f32,
) -> Result<ImageData, JsValue> {
    let width = image_data.width as usize;
    let height = image_data.height as usize;
    let data = image_data.data;

    let mut y_matrix: Vec<Vec<f32>> = vec![vec![0.0; width]; height];
    let mut cb_matrix: Vec<Vec<f32>> = vec![vec![0.0; width]; height];
    let mut cr_matrix: Vec<Vec<f32>> = vec![vec![0.0; width]; height];

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 4;
            let r = data[idx] as f32;
            let g = data[idx + 1] as f32;
            let b = data[idx + 2] as f32;
            y_matrix[y][x] = 0.299 * r + 0.587 * g + 0.114 * b;
            cb_matrix[y][x] = -0.168736 * r - 0.331264 * g + 0.5 * b + 128.0;
            cr_matrix[y][x] = 0.5 * r - 0.418688 * g - 0.081312 * b + 128.0;
        }
    }

    let subsampled_w = width / 2;
    let subsampled_h = height / 2;
    let mut cb_sub: Vec<Vec<f32>> = vec![vec![0.0; subsampled_w]; subsampled_h];
    let mut cr_sub: Vec<Vec<f32>> = vec![vec![0.0; subsampled_w]; subsampled_h];
    for y in 0..subsampled_h {
        for x in 0..subsampled_w {
            cb_sub[y][x] = cb_matrix[y * 2][x * 2];
            cr_sub[y][x] = cr_matrix[y * 2][x * 2];
        }
    }

    let std_quant_matrix: [[u32; 8]; 8] = [
        [16, 11, 10, 16, 24, 40, 51, 61],
        [12, 12, 14, 19, 26, 58, 60, 55],
        [14, 13, 16, 24, 40, 57, 69, 56],
        [14, 17, 22, 29, 51, 87, 80, 62],
        [18, 22, 37, 56, 68, 109, 103, 77],
        [24, 35, 55, 64, 81, 104, 113, 92],
        [49, 64, 78, 87, 103, 121, 120, 101],
        [72, 92, 95, 98, 112, 100, 103, 99],
    ];

    let scale = if quality < 50.0 {
        5000.0 / quality
    } else {
        200.0 - quality * 2.0
    } / 100.0;

    let quant_matrix: [[u32; 8]; 8] = std_quant_matrix.map(|row| {
        row.map(|val| (val as f32 * scale).floor().max(1.0) as u32)
    });

    fn process_blocks(
        channel: Vec<Vec<f32>>,
        width: usize,
        height: usize,
        quant_matrix: [[u32; 8]; 8],
    ) -> Vec<Vec<f32>> {
        let mut processed = vec![vec![0.0; width]; height];

        for i in (0..height).step_by(8) {
            for j in (0..width).step_by(8) {
                let mut block = [[0.0; 8]; 8];
                for u in 0..8 {
                    for v in 0..8 {
                        let y_idx = i + u;
                        let x_idx = j + v;
                        if y_idx < height && x_idx < width {
                            block[u][v] = channel[y_idx][x_idx];
                        }
                    }
                }

                let dct_block = dct2d(block);
                let mut quantized = [[0.0; 8]; 8];
                for u in 0..8 {
                    for v in 0..8 {
                        quantized[u][v] = (dct_block[u][v] / quant_matrix[u][v] as f32).round();
                    }
                }

                for u in 0..8 {
                    for v in 0..8 {
                        quantized[u][v] *= quant_matrix[u][v] as f32;
                    }
                }

                let block_idct = idct2d(quantized);

                for u in 0..8 {
                    for v in 0..8 {
                        let y_idx = i + u;
                        let x_idx = j + v;
                        if y_idx < height && x_idx < width {
                            processed[y_idx][x_idx] = block_idct[u][v];
                        }
                    }
                }
            }
        }
        processed
    }

    fn dct2d(block: [[f32; 8]; 8]) -> [[f32; 8]; 8] {
        let mut dct = [[0.0; 8]; 8];
        for u in 0..8 {
            for v in 0..8 {
                let mut sum = 0.0;
                for x in 0..8 {
                    for y in 0..8 {
                        sum += block[x][y]
                            * ((2 * x + 1) as f32 * u as f32 * std::f32::consts::PI / 16.0).cos()
                            * ((2 * y + 1) as f32 * v as f32 * std::f32::consts::PI / 16.0).cos();
                    }
                }
                let cu = if u == 0 { 1.0 / 2.0f32.sqrt() } else { 1.0 };
                let cv = if v == 0 { 1.0 / 2.0f32.sqrt() } else { 1.0 };
                dct[u][v] = 0.25 * cu * cv * sum;
            }
        }
        dct
    }

    fn idct2d(dct: [[f32; 8]; 8]) -> [[f32; 8]; 8] {
        let mut block = [[0.0; 8]; 8];
        for x in 0..8 {
            for y in 0..8 {
                let mut sum = 0.0;
                for u in 0..8 {
                    for v in 0..8 {
                        let cu = if u == 0 { 1.0 / 2.0f32.sqrt() } else { 1.0 };
                        let cv = if v == 0 { 1.0 / 2.0f32.sqrt() } else { 1.0 };
                        sum += cu * cv * dct[u][v]
                            * ((2 * x + 1) as f32 * u as f32 * std::f32::consts::PI / 16.0).cos()
                            * ((2 * y + 1) as f32 * v as f32 * std::f32::consts::PI / 16.0).cos();
                    }
                }
                block[x][y] = 0.25 * sum;
            }
        }
        block
    }

    let y_processed = process_blocks(y_matrix, width, height, quant_matrix);
    let cb_processed = process_blocks(cb_sub.clone(), subsampled_w, subsampled_h, quant_matrix);
    let cr_processed = process_blocks(cr_sub, subsampled_w, subsampled_h, quant_matrix);

    let mut cb_up: Vec<Vec<f32>> = vec![vec![0.0; width]; height];
    let mut cr_up: Vec<Vec<f32>> = vec![vec![0.0; width]; height];
    for y in 0..height {
        for x in 0..width {
            let src_y = y / 2;
            let src_x = x / 2;
            if src_y < subsampled_h && src_x < subsampled_w {
                cb_up[y][x] = cb_processed[src_y][src_x];
                cr_up[y][x] = cr_processed[src_y][src_x];
            }
        }
    }

    let mut output_data = vec![0; width * height * 4];
    for y in 0..height {
        for x in 0..width {
            let y_val = y_processed[y][x];
            let cb_val = cb_up[y][x] - 128.0;
            let cr_val = cr_up[y][x] - 128.0;
            let r = y_val + 1.402 * cr_val;
            let g = y_val - 0.344136 * cb_val - 0.714136 * cr_val;
            let b = y_val + 1.772 * cb_val;
            let idx = (y * width + x) * 4;
            output_data[idx] = r.round().clamp(0.0, 255.0) as u8;
            output_data[idx + 1] = g.round().clamp(0.0, 255.0) as u8;
            output_data[idx + 2] = b.round().clamp(0.0, 255.0) as u8;
            output_data[idx + 3] = 255;
        }
    }

    Ok(ImageData {
        width: width as u32,
        height: height as u32,
        data: output_data,
    })
}