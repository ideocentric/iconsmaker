pub mod compose;
pub mod effects;
pub mod rasterize;
pub mod squircle;

/// RGBA pixel buffer produced by rasterizing an SVG.
///
/// Pixels are stored in **premultiplied** RGBA format (tiny-skia's native output).
/// Call `to_straight_alpha()` before PNG, ICO, or ICNS encoding.
#[derive(Clone)]
pub struct RasterBuffer {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl RasterBuffer {
    /// Returns a straight-alpha copy of the pixel data (un-premultiplied).
    /// Fully-transparent and fully-opaque pixels are fast-pathed.
    pub fn to_straight_alpha(&self) -> Vec<u8> {
        let mut out = self.data.clone();
        for px in out.chunks_exact_mut(4) {
            let a = px[3];
            if a > 0 && a < 255 {
                let scale = 255.0 / a as f32;
                px[0] = (px[0] as f32 * scale).round().min(255.0) as u8;
                px[1] = (px[1] as f32 * scale).round().min(255.0) as u8;
                px[2] = (px[2] as f32 * scale).round().min(255.0) as u8;
            }
        }
        out
    }
}