# Compress JPEG

![Version](https://img.shields.io/npm/v/compress-jpeg)
![Downloads](https://img.shields.io/npm/dw/compress-jpeg)
![License](https://img.shields.io/npm/l/compress-jpeg)

A lightweight WebAssembly-powered JPEG-like compressor written in Rust, exposed as an easy-to-use JavaScript API. This package performs a simplified JPEG pipeline entirely in WASM, including:

-   RGB → YCbCr conversion
-   4:2:0 chroma subsampling
-   8×8 DCT + quantization
-   Inverse DCT
-   Reconstruction to RGBA

The output is not a `.jpg` file, but a visually compressed `ImageData` that simulates JPEG compression artifacts—including blockiness, color loss, and ringing—directly in the browser.

## 📋 Table of Contents

-   [Features](#-features)
-   [Installation](#-installation)
-   [Usage](#-usage)
-   [License](#-license)
-   [Contact](#-contact)

## ✨ Features

-   Fast WebAssembly image compression (Rust + wasm-bindgen)
-   Fully controllable compression strength (`0.0 → 1.0`)
-   Implements a full JPEG-style DCT/IDCT pipeline
-   Produces visible JPEG artifacts at higher compression levels
-   Works directly with Canvas `ImageData`
-   Zero dependencies — tiny package size
-   Browser-friendly and easy to use
-   Simple API: `compress_jpeg(imageData: ImageData, compression: number): ImageData`

## 🔧 Installation

```bash
npm install compress-jpeg
# or
yarn add compress-jpeg
```

## 🚀 Usage

```typescript
import init, { compress_jpeg } from "compress-jpeg";

async function run() {
    await init(); // initialize WASM module

    const canvas = document.getElementById("my-canvas") as HTMLCanvasElement;
    const ctx = canvas.getContext("2d")!;
    const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);

    /**
     * Compression strength (0.0 → 1.0)
     *
     * - 0.0 = no compression (highest quality)
     * - 1.0 = strongest compression (lowest quality, heavy artifacts)
     *
     * Recommended ranges:
     * - 0.7–1.0 → strong blockiness / heavy JPEG artifacts
     * - 0.3–0.7 → medium compression
     * - 0.0–0.3 → light compression / near-lossless
     */
    const compression = 0.4;

    const output = compress_jpeg(imageData, compression);

    // Draw result onto canvas
    ctx.putImageData(output, 0, 0);
}
```

## 📜 License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## 📧 Contact

For inquiries or more information, you can reach out to us at [ganemedelabs@gmail.com](mailto:ganemedelabs@gmail.com).
