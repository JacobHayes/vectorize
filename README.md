# Vectorize

A browser-based tool that converts raster images to SVG vector graphics. All processing happens locally in your browser using WebAssembly—no images are uploaded to any server.

## Features

- **Supported formats**: PNG, JPEG, GIF, WebP
- **Presets**: Default, Black & White, Poster, Photo
- **Advanced controls**: Color mode, speckle filtering, color precision, corner threshold, segment length, splice threshold
- **Instant preview**: See the vectorized result immediately
- **Download**: Export your SVG with one click

## Usage

```bash
./scripts/serve.sh
```

Then open http://localhost:8080 in your browser.

The script will:
1. Install `wasm-pack` if not present
2. Build the Rust code to WebAssembly
3. Start a local server on port 8080

## Development

**Prerequisites**: Rust toolchain

**Build manually**:
```bash
wasm-pack build --target web
```

**Project structure**:
- `src/lib.rs` - Rust/WASM vectorization logic using vtracer
- `index.html` - Frontend UI and JavaScript
- `pkg/` - Generated WASM build output

## How It Works

The application uses [vtracer](https://github.com/visioncortex/vtracer) to convert raster images to vector paths. The Rust code is compiled to WebAssembly via `wasm-pack`, allowing the vectorization algorithm to run at near-native speed directly in the browser.
