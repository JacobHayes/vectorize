# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Vectorize is a WebAssembly-based web application that converts raster images (PNG, JPEG, GIF, WebP) to SVG files entirely in the browser. No server-side processing occurs.

## Build Commands

```bash
# Build and serve (installs wasm-pack if needed, builds WASM, starts server on :8080)
scripts/serve.sh

# Manual build only
wasm-pack build --target web
```

Build output goes to `pkg/` directory containing JavaScript bindings and the compiled WASM binary.

## Architecture

**Rust/WASM Backend (`src/lib.rs`):**
- `init()` - Entry point, sets up panic hooks
- `convert_png_to_svg(image_bytes, options)` - Main conversion function exposed to JavaScript
- `ConversionOptions` - Deserialized from JS, configures vtracer parameters (preset, color_mode, filter_speckle, etc.)
- `ConversionResult` - Returns SVG string, width, and height to JavaScript

**Frontend (`index.html`):**
- Single-page application with vanilla ES6 JavaScript
- Imports WASM module, handles file upload/drag-drop, renders SVG preview
- Provides preset selector (BW/Poster/Photo) and advanced parameter controls

**Key Dependencies:**
- `vtracer` - Core vectorization algorithm
- `wasm-bindgen` - JavaScript/Rust interoperability
- `image` - Multi-format image decoding
- `serde-wasm-bindgen` - Options serialization from JavaScript
