use wasm_bindgen::prelude::*;
use web_sys::ImageData as BrowserImageData;

/// Compress an ImageData using a simplified JPEG-style pipeline.
///
/// **Parameters:**
/// - `image_data`: The RGBA ImageData to compress.
/// - `compression`: A value from 0.0–1.0:
///     - 0.0 = no compression (highest quality)
///     - 1.0 = strongest compression (lowest quality)
///
/// **Returns:**
/// A new `ImageData` object containing the visually compressed pixels.
#[wasm_bindgen]
pub fn compress_jpeg(
    image_data: BrowserImageData,
    compression: f32,
) -> Result<BrowserImageData, JsValue> {
    if compression <= 0.0 {
        let width = image_data.width();
        let height = image_data.height();
        let data = image_data.data();

        return BrowserImageData::new_with_u8_clamped_array_and_sh(
            wasm_bindgen::Clamped(&data.to_vec()),
            width,
            height,
        );
    }

    let width = image_data.width() as usize;
    let height = image_data.height() as usize;

    let data_vec: Vec<u8> = image_data.data().to_vec();

    let mut y_matrix = vec![vec![0.0; width]; height];
    let mut cb_matrix = vec![vec![0.0; width]; height];
    let mut cr_matrix = vec![vec![0.0; width]; height];

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 4;
            let r = data_vec[idx] as f32;
            let g = data_vec[idx + 1] as f32;
            let b = data_vec[idx + 2] as f32;

            y_matrix[y][x] = 0.299 * r + 0.587 * g + 0.114 * b;
            cb_matrix[y][x] = -0.168736 * r - 0.331264 * g + 0.5 * b + 128.0;
            cr_matrix[y][x] = 0.5 * r - 0.418688 * g - 0.081312 * b + 128.0;
        }
    }

    let subsampled_w = width / 2;
    let subsampled_h = height / 2;

    let mut cb_sub = vec![vec![0.0; subsampled_w]; subsampled_h];
    let mut cr_sub = vec![vec![0.0; subsampled_w]; subsampled_h];

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

    let c = compression.clamp(0.0, 1.0);

    const MAX_FACTOR: f32 = 20.0;

    let scale_factor = 1.0 + c * MAX_FACTOR;

    let quant_matrix: [[u32; 8]; 8] = std_quant_matrix.map(|row| {
        row.map(|v| (v as f32 * scale_factor).floor().max(1.0) as u32)
    });

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
                let cu = if u == 0 { 1.0 / 2.0_f32.sqrt() } else { 1.0 };
                let cv = if v == 0 { 1.0 / 2.0_f32.sqrt() } else { 1.0 };
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
                        let cu = if u == 0 { 1.0 / 2.0_f32.sqrt() } else { 1.0 };
                        let cv = if v == 0 { 1.0 / 2.0_f32.sqrt() } else { 1.0 };
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

    fn process_blocks(
        channel: Vec<Vec<f32>>,
        width: usize,
        height: usize,
        quant: [[u32; 8]; 8],
    ) -> Vec<Vec<f32>> {
        let mut out = vec![vec![0.0; width]; height];

        for by in (0..height).step_by(8) {
            for bx in (0..width).step_by(8) {

                let mut block = [[0.0; 8]; 8];
                for u in 0..8 {
                    for v in 0..8 {
                        let y = by + u;
                        let x = bx + v;
                        if y < height && x < width {
                            block[u][v] = channel[y][x];
                        }
                    }
                }

                let dct = dct2d(block);

                let mut q = [[0.0; 8]; 8];
                for u in 0..8 {
                    for v in 0..8 {
                        q[u][v] = (dct[u][v] / quant[u][v] as f32).round();
                        q[u][v] *= quant[u][v] as f32;
                    }
                }

                let idct = idct2d(q);

                for u in 0..8 {
                    for v in 0..8 {
                        let y = by + u;
                        let x = bx + v;
                        if y < height && x < width {
                            out[y][x] = idct[u][v];
                        }
                    }
                }
            }
        }

        out
    }

    let y_proc = process_blocks(y_matrix, width, height, quant_matrix);
    let cb_proc = process_blocks(cb_sub.clone(), subsampled_w, subsampled_h, quant_matrix);
    let cr_proc = process_blocks(cr_sub,   subsampled_w, subsampled_h, quant_matrix);

    let mut cb_up = vec![vec![0.0; width]; height];
    let mut cr_up = vec![vec![0.0; width]; height];

    for y in 0..height {
        for x in 0..width {
            let sy = y / 2;
            let sx = x / 2;
            cb_up[y][x] = cb_proc[sy][sx];
            cr_up[y][x] = cr_proc[sy][sx];
        }
    }

    let mut out = vec![0u8; width * height * 4];

    for y in 0..height {
        for x in 0..width {
            let y_value = y_proc[y][x];
            let cb = cb_up[y][x] - 128.0;
            let cr = cr_up[y][x] - 128.0;

            let r = y_value + 1.402 * cr;
            let g = y_value - 0.344136 * cb - 0.714136 * cr;
            let b = y_value + 1.772 * cb;

            let idx = (y * width + x) * 4;
            out[idx]     = r.clamp(0.0, 255.0) as u8;
            out[idx + 1] = g.clamp(0.0, 255.0) as u8;
            out[idx + 2] = b.clamp(0.0, 255.0) as u8;
            out[idx + 3] = 255;
        }
    }

    BrowserImageData::new_with_u8_clamped_array_and_sh(
        wasm_bindgen::Clamped(&out[..]),
        width as u32,
        height as u32,
    )
}
