use image::{codecs::png::PngEncoder, ColorType, ImageEncoder, RgbaImage};
use vrcx_0_vr_overlay::{
    MainSurfaceModel, RgbaFrame, SlintHmdRenderer, SlintWristRenderer, WristSurfaceModel,
};

pub struct RenderedPng {
    pub bytes: Vec<u8>,
}

pub struct DevtoolRenderer {
    wrist: SlintWristRenderer,
    hmd: SlintHmdRenderer,
}

impl DevtoolRenderer {
    pub fn new() -> Self {
        Self {
            wrist: SlintWristRenderer::new(),
            hmd: SlintHmdRenderer::new(),
        }
    }

    pub fn main_png(&mut self, model: &MainSurfaceModel) -> Result<RenderedPng, String> {
        let frame = self.hmd.render(model)?;
        frame_png(frame).map(RenderedPng::new)
    }

    pub fn wrist_png(&mut self, model: &WristSurfaceModel) -> Result<RenderedPng, String> {
        let frame = self.wrist.render(model)?;
        frame_png(frame).map(RenderedPng::new)
    }
}

impl RenderedPng {
    fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
}

impl Default for DevtoolRenderer {
    fn default() -> Self {
        Self::new()
    }
}

const BACKDROPS: [u8; 3] = [0, 110, 235];

pub fn backdrop_sheet_png(png: &[u8]) -> Result<Vec<u8>, String> {
    let overlay = image::load_from_memory(png)
        .map_err(|error| format!("decode PNG failed: {error}"))?
        .into_rgba8();
    let (width, height) = overlay.dimensions();
    let mut sheet = RgbaImage::new(width * BACKDROPS.len() as u32, height);
    for (tile, level) in BACKDROPS.iter().enumerate() {
        let offset = tile as u32 * width;
        for (x, y, pixel) in overlay.enumerate_pixels() {
            let alpha = f32::from(pixel[3]) / 255.0;
            let blended: [u8; 3] = std::array::from_fn(|channel| {
                (f32::from(pixel[channel]) + f32::from(*level) * (1.0 - alpha)).round() as u8
            });
            sheet.put_pixel(
                offset + x,
                y,
                image::Rgba([blended[0], blended[1], blended[2], 255]),
            );
        }
    }
    let mut encoded = Vec::new();
    PngEncoder::new(&mut encoded)
        .write_image(
            sheet.as_raw(),
            sheet.width(),
            sheet.height(),
            ColorType::Rgba8.into(),
        )
        .map_err(|error| format!("encode PNG failed: {error}"))?;
    Ok(encoded)
}

pub fn frame_png(frame: RgbaFrame) -> Result<Vec<u8>, String> {
    if !frame.is_valid_len() {
        return Err(format!(
            "invalid frame length for {}x{}",
            frame.size.width, frame.size.height
        ));
    }
    let mut png = Vec::new();
    PngEncoder::new(&mut png)
        .write_image(
            &frame.data,
            frame.size.width,
            frame.size.height,
            ColorType::Rgba8.into(),
        )
        .map_err(|error| format!("encode PNG failed: {error}"))?;
    Ok(png)
}
