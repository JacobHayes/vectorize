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
    pub output_width: Option<u32>,
    pub output_height: Option<u32>,
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
            output_width: None,
            output_height: None,
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
    output_width: u32,
    output_height: u32,
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

    #[wasm_bindgen(getter)]
    pub fn output_width(&self) -> u32 {
        self.output_width
    }

    #[wasm_bindgen(getter)]
    pub fn output_height(&self) -> u32 {
        self.output_height
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

    let raw = svg_file.to_string();

    let opt_w = options.output_width.filter(|&v| v > 0);
    let opt_h = options.output_height.filter(|&v| v > 0);
    let (out_w, out_h) = match (opt_w, opt_h) {
        (Some(w), Some(h)) => (w, h),
        (Some(w), None) => (w, (w as f64 * height as f64 / width as f64).round() as u32),
        (None, Some(h)) => ((h as f64 * width as f64 / height as f64).round() as u32, h),
        (None, None) => (width, height),
    };

    let svg = raw
        .replacen(
            &format!("width=\"{}\"", width),
            &format!("width=\"{}\"", out_w),
            1,
        )
        .replacen(
            &format!("height=\"{}\"", height),
            &format!("height=\"{}\"", out_h),
            1,
        );

    // Inject viewBox using original dims so coordinate space matches path data
    let viewbox = format!(" viewBox=\"0 0 {} {}\"", width, height);
    let svg = match svg.find("<svg") {
        Some(idx) => {
            let mut s = String::with_capacity(svg.len() + viewbox.len());
            s.push_str(&svg[..idx + 4]);
            s.push_str(&viewbox);
            s.push_str(&svg[idx + 4..]);
            s
        }
        None => svg,
    };

    Ok(ConversionResult { svg, width, height, output_width: out_w, output_height: out_h })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_image(w: usize, h: usize) -> ColorImage {
        ColorImage {
            pixels: vec![255; w * h * 4],
            width: w,
            height: h,
        }
    }

    fn convert_with_options(w: usize, h: usize, out_w: Option<u32>, out_h: Option<u32>) -> String {
        let color_image = make_test_image(w, h);
        let config = Config::default();
        let svg_file = convert(color_image, config).unwrap();
        let raw = svg_file.to_string();

        let width = w as u32;
        let height = h as u32;
        let opt_w = out_w.filter(|&v| v > 0);
        let opt_h = out_h.filter(|&v| v > 0);
        let (out_w, out_h) = match (opt_w, opt_h) {
            (Some(w), Some(h)) => (w, h),
            (Some(w), None) => (w, (w as f64 * height as f64 / width as f64).round() as u32),
            (None, Some(h)) => ((h as f64 * width as f64 / height as f64).round() as u32, h),
            (None, None) => (width, height),
        };

        let svg = raw
            .replacen(
                &format!("width=\"{}\"", width),
                &format!("width=\"{}\"", out_w),
                1,
            )
            .replacen(
                &format!("height=\"{}\"", height),
                &format!("height=\"{}\"", out_h),
                1,
            );

        let viewbox = format!(" viewBox=\"0 0 {} {}\"", width, height);
        match svg.find("<svg") {
            Some(idx) => {
                let mut s = String::with_capacity(svg.len() + viewbox.len());
                s.push_str(&svg[..idx + 4]);
                s.push_str(&viewbox);
                s.push_str(&svg[idx + 4..]);
                s
            }
            None => svg,
        }
    }

    #[test]
    fn vtracer_output_contains_expected_attributes() {
        let img = make_test_image(10, 8);
        let svg_file = convert(img, Config::default()).unwrap();
        let raw = svg_file.to_string();
        assert!(raw.contains("<svg"), "SVG must contain <svg tag");
        assert!(raw.contains("width=\"10\""), "SVG must contain width=\"10\", got: {}", &raw[..200.min(raw.len())]);
        assert!(raw.contains("height=\"8\""), "SVG must contain height=\"8\", got: {}", &raw[..200.min(raw.len())]);
        assert!(!raw.contains("viewBox"), "vtracer should not emit viewBox");
    }

    #[test]
    fn default_dimensions_preserved() {
        let svg = convert_with_options(10, 8, None, None);
        assert!(svg.contains("width=\"10\""));
        assert!(svg.contains("height=\"8\""));
        assert!(svg.contains("viewBox=\"0 0 10 8\""));
    }

    #[test]
    fn override_both_dimensions() {
        let svg = convert_with_options(10, 8, Some(800), Some(600));
        assert!(svg.contains("width=\"800\""), "width should be overridden to 800");
        assert!(svg.contains("height=\"600\""), "height should be overridden to 600");
        assert!(svg.contains("viewBox=\"0 0 10 8\""), "viewBox must use original dims");
        assert!(!svg.contains("width=\"10\""), "original width should be gone");
        assert!(!svg.contains("height=\"8\""), "original height should be gone");
    }

    #[test]
    fn override_only_width_scales_height() {
        // 10x8 image, width=400 -> height = 400 * 8/10 = 320
        let svg = convert_with_options(10, 8, Some(400), None);
        assert!(svg.contains("width=\"400\""));
        assert!(svg.contains("height=\"320\""), "height should scale proportionally");
        assert!(svg.contains("viewBox=\"0 0 10 8\""));
    }

    #[test]
    fn override_only_height_scales_width() {
        // 10x8 image, height=300 -> width = 300 * 10/8 = 375
        let svg = convert_with_options(10, 8, None, Some(300));
        assert!(svg.contains("width=\"375\""), "width should scale proportionally");
        assert!(svg.contains("height=\"300\""));
        assert!(svg.contains("viewBox=\"0 0 10 8\""));
    }

    #[test]
    fn zero_override_falls_back_to_original() {
        let svg = convert_with_options(10, 8, Some(0), Some(0));
        assert!(svg.contains("width=\"10\""));
        assert!(svg.contains("height=\"8\""));
    }
}
