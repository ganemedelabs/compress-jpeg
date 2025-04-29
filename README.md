# Compress JPEG

![npm](https://img.shields.io/npm/v/compress-jpeg)
![npm](https://img.shields.io/npm/dw/compress-jpeg)
![License](https://img.shields.io/npm/l/compress-jpeg)

A high-performance **WASM** module that simulates JPEG compression artifacts on raw image data, designed to be used easily in JavaScript or TypeScript environments via an npm package.

## 📋 Table of Contents

-   [Features](#-features)
-   [Installation](#-installation)
-   [Usage](#-usage)
-   [License](#-license)
-   [Contact](#-contact)

## ✨ Features

-   **Fast** and **portable** through WebAssembly.
-   Supports customizable **compression quality**.
-   Accepts and returns raw **RGBA pixel data**.
-   Pure Rust core with zero dependencies beyond `wasm-bindgen`.

## 🔧 Installation

Install the package via npm:

```bash
npm install compress-jpeg
```

## 🚀 Usage

Here's an example of how to integrate **compress-jpeg** with plain TypeScript in the browser. This snippet shows:

1. Loading an image file via an HTML `<input>`.
2. Drawing it to an off-screen canvas and extracting pixel data.
3. Passing the data to the WASM function.
4. Rendering the compressed output back onto a visible canvas.

```html
<!-- index.html -->
<!DOCTYPE html>
<html lang="en">
    <head>
        <meta charset="UTF-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <title>WASM JPEG Simulator Demo</title>
    </head>
    <body>
        <!-- File input and canvases -->
        <input id="fileInput" type="file" accept="image/*" />
        <canvas id="originalCanvas" style="display:none;"></canvas>
        <canvas id="processedCanvas"></canvas>

        <script type="module" src="main.ts"></script>
    </body>
</html>
```

```typescript
// main.ts
import init, { ImageData as RustImageData, compress_jpeg } from "compress-jpeg";

const fileInput = document.getElementById("fileInput") as HTMLInputElement;
const originalCanvas = document.getElementById("originalCanvas") as HTMLCanvasElement;
const processedCanvas = document.getElementById("processedCanvas") as HTMLCanvasElement;

fileInput.addEventListener("change", async () => {
    const file = fileInput.files?.[0];
    if (!file) return;

    // Initialize the WASM module with an explicit path
    await init("/compress_jpeg_bg.wasm");

    // Load image into an off-screen canvas
    const img = new Image();
    const reader = new FileReader();
    reader.onload = () => {
        img.src = reader.result as string;
    };
    reader.readAsDataURL(file);

    img.onload = () => {
        originalCanvas.width = img.width;
        originalCanvas.height = img.height;
        const ctx = originalCanvas.getContext("2d")!;
        ctx.drawImage(img, 0, 0);

        // Extract raw pixel data
        const browserImageData = ctx.getImageData(0, 0, img.width, img.height);
        const rustInput = new RustImageData(
            browserImageData.width,
            browserImageData.height,
            new Uint8ClampedArray(browserImageData.data)
        );

        // Simulate JPEG compression at quality=30
        const output = compress_jpeg(rustInput, 30);

        // Convert back to browser ImageData
        const processedData = new ImageData(output.data(), output.width(), output.height());

        // Draw on visible canvas
        processedCanvas.width = output.width();
        processedCanvas.height = output.height();
        processedCanvas.getContext("2d")!.putImageData(processedData, 0, 0);
    };
});
```

## 📜 License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## 📧 Contact

For inquiries or more information, you can reach out to us at [ganemedelabs@gmail.com](mailto:ganemedelabs@gmail.com).
