use super::squircle::squircle_mask;
use super::RasterBuffer;

/// Apply the squircle mask to `buf`, making every pixel outside the boundary
/// fully transparent.
///
/// The mask is generated analytically from the superellipse formula with
/// the given power `n` — no external SVG file is needed.  Anti-aliased at
/// ±1 pixel from the boundary.
///
/// Call this AFTER `apply_squircle_depth` (if used) so depth shading is
/// clipped cleanly by the squircle edge.
pub fn apply_squircle_mask(buf: &mut RasterBuffer, n: f32) {
    let mask = squircle_mask(buf.width, n);

    // mask alpha == 0   → inside squircle  → keep icon pixel unchanged
    // mask alpha == 255 → outside squircle → zero icon pixel to transparent
    // intermediate      → anti-aliased edge → blend proportionally
    //
    // Pixels are premultiplied RGBA, so scaling all four channels by the same
    // keep-fraction is the correct compositing operation.
    for (icon_px, mask_px) in buf.data.chunks_mut(4).zip(mask.data.chunks(4)) {
        let mask_alpha = mask_px[3];
        if mask_alpha > 0 {
            let keep = 255u16 - mask_alpha as u16;
            for ch in icon_px.iter_mut() {
                *ch = ((*ch as u16 * keep) / 255) as u8;
            }
        }
    }
}