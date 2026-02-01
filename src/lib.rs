use wasm_bindgen::prelude::*;
use web_sys::ImageData as BrowserImageData;
use std::f32::consts::PI;

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
    let width = image_data.width() as usize;
    let height = image_data.height() as usize;
    
    if width == 0 || height == 0 {
        return Err(JsValue::from_str("INVALID_DIMENSIONS"));
    }

    let c_factor = compression.clamp(0.0, 1.0);

    if c_factor <= 0.0 {
        return Ok(image_data);
    }

    let data_vec = image_data.data();
    if data_vec.len() != width * height * 4 {
        return Err(JsValue::from_str("BUFFER_MISMATCH"));
    }

    let mut y_matrix = vec![0.0; width * height];
    let mut cb_matrix = vec![0.0; width * height];
    let mut cr_matrix = vec![0.0; width * height];

    for y in 0..height {
        for x in 0..width {
            let i = (y * width + x) * 4;
            let r = *data_vec.get(i).unwrap_or(&0) as f32;
            let g = *data_vec.get(i + 1).unwrap_or(&0) as f32;
            let b = *data_vec.get(i + 2).unwrap_or(&0) as f32;

            let m_i = y * width + x;
            y_matrix[m_i] = 0.299 * r + 0.587 * g + 0.114 * b;
            cb_matrix[m_i] = -0.168736 * r - 0.331264 * g + 0.5 * b + 128.0;
            cr_matrix[m_i] = 0.5 * r - 0.418688 * g - 0.081312 * b + 128.0;
        }
    }

    let sub_w = (width + 1) / 2;
    let sub_h = (height + 1) / 2;
    let mut cb_sub = vec![0.0; sub_w * sub_h];
    let mut cr_sub = vec![0.0; sub_w * sub_h];

    for y in 0..sub_h {
        for x in 0..sub_w {
            let src_y = (y * 2).min(height - 1);
            let src_x = (x * 2).min(width - 1);
            let src_idx = src_y * width + src_x;
            let dst_idx = y * sub_w + x;
            cb_sub[dst_idx] = cb_matrix[src_idx];
            cr_sub[dst_idx] = cr_matrix[src_idx];
        }
    }

    let std_quant: [[u32; 8]; 8] = [
        [16, 11, 10, 16, 24, 40, 51, 61], [12, 12, 14, 19, 26, 58, 60, 55],
        [14, 13, 16, 24, 40, 57, 69, 56], [14, 17, 22, 29, 51, 87, 80, 62],
        [18, 22, 37, 56, 68, 109, 103, 77], [24, 35, 55, 64, 81, 104, 113, 92],
        [49, 64, 78, 87, 103, 121, 120, 101], [72, 92, 95, 98, 112, 100, 103, 99],
    ];
    let scale = 1.0 + c_factor * 20.0;
    let q_mat: [[u32; 8]; 8] = std_quant.map(|r| r.map(|v| (v as f32 * scale).floor().max(1.0) as u32));

    let y_res = process_blocks(&y_matrix, width, height, &q_mat);
    let cb_res = process_blocks(&cb_sub, sub_w, sub_h, &q_mat);
    let cr_res = process_blocks(&cr_sub, sub_w, sub_h, &q_mat);

    let mut output = vec![0u8; width * height * 4];
    for y in 0..height {
        for x in 0..width {
            let sy = (y / 2).min(sub_h - 1);
            let sx = (x / 2).min(sub_w - 1);

            let y_v = y_res[y * width + x];
            let cb = cb_res[sy * sub_w + sx] - 128.0;
            let cr = cr_res[sy * sub_w + sx] - 128.0;

            let r = (y_v + 1.402 * cr).clamp(0.0, 255.0) as u8;
            let g = (y_v - 0.344136 * cb - 0.714136 * cr).clamp(0.0, 255.0) as u8;
            let b = (y_v + 1.772 * cb).clamp(0.0, 255.0) as u8;

            let idx = (y * width + x) * 4;
            output[idx] = r;
            output[idx + 1] = g;
            output[idx + 2] = b;
            output[idx + 3] = 255;
        }
    }

    BrowserImageData::new_with_u8_clamped_array_and_sh(
        wasm_bindgen::Clamped(&output),
        width as u32,
        height as u32,
    )
}

fn process_blocks(input: &[f32], w: usize, h: usize, q: &[[u32; 8]; 8]) -> Vec<f32> {
    let mut out = vec![0.0; w * h];
    for by in (0..h).step_by(8) {
        for bx in (0..w).step_by(8) {
            let mut block = [[0.0; 8]; 8];
            for u in 0..8 {
                for v in 0..8 {
                    let py = by + u;
                    let px = bx + v;
                    if py < h && px < w {
                        block[u][v] = input[py * w + px];
                    } else {
                        let py_edge = py.min(h - 1);
                        let px_edge = px.min(w - 1);
                        block[u][v] = input[py_edge * w + px_edge];
                    }
                }
            }

            let processed = idct2d(quantize(dct2d(block), q));

            for u in 0..8 {
                for v in 0..8 {
                    let py = by + u;
                    let px = bx + v;
                    if py < h && px < w {
                        out[py * w + px] = processed[u][v];
                    }
                }
            }
        }
    }
    out
}

fn quantize(mut dct: [[f32; 8]; 8], q: &[[u32; 8]; 8]) -> [[f32; 8]; 8] {
    for u in 0..8 {
        for v in 0..8 {
            dct[u][v] = (dct[u][v] / q[u][v] as f32).round() * q[u][v] as f32;
        }
    }
    dct
}

fn dct2d(block: [[f32; 8]; 8]) -> [[f32; 8]; 8] {
    let mut dct = [[0.0; 8]; 8];
    for u in 0..8 {
        for v in 0..8 {
            let mut sum = 0.0;
            for x in 0..8 {
                for y in 0..8 {
                    sum += block[x][y]
                        * ((2 * x + 1) as f32 * u as f32 * PI / 16.0).cos()
                        * ((2 * y + 1) as f32 * v as f32 * PI / 16.0).cos();
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
                        * ((2 * x + 1) as f32 * u as f32 * PI / 16.0).cos()
                        * ((2 * y + 1) as f32 * v as f32 * PI / 16.0).cos();
                }
            }
            block[x][y] = 0.25 * sum;
        }
    }
    block
}