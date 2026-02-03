use image::GenericImageView;
use serde::Deserialize;
use vtracer::{ColorImage, Config, convert};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[derive(Deserialize)]
#[serde(default)]
pub struct ConversionOptions {
    pub preset: String,
    pub color_mode: String,
    pub filter_speckle: u32,
    pub color_precision: u32,
    pub corner_threshold: u32,
    pub segment_length: f64,
    pub splice_threshold: u32,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            preset: String::new(),
            color_mode: String::new(),
            filter_speckle: 4,
            color_precision: 6,
            corner_threshold: 60,
            segment_length: 4.0,
            splice_threshold: 45,
        }
    }
}

impl ConversionOptions {
    fn apply_preset(&mut self) {
        match self.preset.as_str() {
            "bw" => {
                self.color_mode = "binary".to_string();
                self.filter_speckle = 4;
                self.color_precision = 6;
                self.corner_threshold = 60;
                self.segment_length = 4.0;
                self.splice_threshold = 45;
            }
            "poster" => {
                self.color_mode = "color".to_string();
                self.filter_speckle = 10;
                self.color_precision = 8;
                self.corner_threshold = 60;
                self.segment_length = 4.0;
                self.splice_threshold = 45;
            }
            "photo" => {
                self.color_mode = "color".to_string();
                self.filter_speckle = 4;
                self.color_precision = 4;
                self.corner_threshold = 90;
                self.segment_length = 2.0;
                self.splice_threshold = 45;
            }
            _ => {}
        }
    }
}

#[wasm_bindgen]
pub struct ConversionResult {
    svg: String,
    width: u32,
    height: u32,
}

#[wasm_bindgen]
impl ConversionResult {
    #[wasm_bindgen(getter)]
    pub fn svg(&self) -> String {
        self.svg.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.height
    }
}

#[wasm_bindgen]
pub fn convert_png_to_svg(
    image_bytes: &[u8],
    options_js: JsValue,
) -> Result<ConversionResult, JsValue> {
    let mut options: ConversionOptions = serde_wasm_bindgen::from_value(options_js)
        .map_err(|e| JsValue::from_str(&format!("Invalid options: {}", e)))?;

    options.apply_preset();

    let img = image::load_from_memory(image_bytes)
        .map_err(|e| JsValue::from_str(&format!("Failed to decode image: {}", e)))?;

    let (width, height) = img.dimensions();
    let rgba = img.to_rgba8();
    let pixels = rgba.into_raw();

    let color_image = ColorImage {
        pixels,
        width: width as usize,
        height: height as usize,
    };

    let config = Config {
        color_mode: match options.color_mode.as_str() {
            "binary" => vtracer::ColorMode::Binary,
            _ => vtracer::ColorMode::Color,
        },
        filter_speckle: options.filter_speckle as usize,
        color_precision: options.color_precision as i32,
        corner_threshold: options.corner_threshold as i32,
        length_threshold: options.segment_length,
        splice_threshold: options.splice_threshold as i32,
        ..Default::default()
    };

    let svg_file = convert(color_image, config)
        .map_err(|e| JsValue::from_str(&format!("Vectorization failed: {}", e)))?;

    Ok(ConversionResult {
        svg: svg_file.to_string(),
        width,
        height,
    })
}
