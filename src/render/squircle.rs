use super::RasterBuffer;

/// Generate a squircle mask using the superellipse formula:
///
/// ```text
///   f(x, y) = |dx / r|^n  +  |dy / r|^n
/// ```
///
/// where `dx`, `dy` are pixel offsets from the icon centre and `r` is the
/// half-size.  Pixels where `f < 1` are inside (alpha = 0); `f > 1` outside
/// (alpha = 255).  The boundary is anti-aliased with a ±1-pixel soft edge
/// derived from the exact gradient magnitude of `f`, giving uniform softness
/// regardless of local curvature.
///
/// ## Mask convention (matches `apply_shape_mask`)
/// * `alpha = 0`   → inside the squircle — the icon's visible area
/// * `alpha = 255` → outside the squircle — clipped away by masking
///
/// ## Choosing `n`
/// | `n` | Shape |
/// |---|---|
/// | 2 | Circle |
/// | 4 | Classic mathematical squircle (quite round) |
/// | **5** | **Visually close to macOS app icon corners (default)** |
/// | 8 | Nearly rectangular with soft corners |
pub fn squircle_mask(size: u32, n: f32) -> RasterBuffer {
    let w  = size as usize;
    let h  = size as usize;
    let cx = size as f32 * 0.5;
    let cy = size as f32 * 0.5;
    let r  = size as f32 * 0.5;

    let mut data = vec![0u8; w * h * 4];

    for y in 0..h {
        for x in 0..w {
            let abs_dx = ((x as f32 - cx) / r).abs();
            let abs_dy = ((y as f32 - cy) / r).abs();

            // Superellipse value: 0 at centre, 1 on boundary, >1 outside.
            let d = abs_dx.powf(n) + abs_dy.powf(n);

            // Gradient magnitude of f in pixel space.
            // ∂f/∂x = n · |dx/r|^(n-1) / r  (chain rule)
            let gx  = n * abs_dx.powf(n - 1.0) / r;
            let gy  = n * abs_dy.powf(n - 1.0) / r;
            let grd = (gx * gx + gy * gy).sqrt().max(1e-6);

            // Signed pixel distance from the boundary (+  = outside, − = inside).
            // Dividing (d − 1) by the gradient converts normalised units → pixels.
            let px_dist = (d - 1.0) / grd;

            let alpha = if px_dist < -1.0 {
                0u8   // fully inside
            } else if px_dist > 1.0 {
                255u8 // fully outside
            } else {
                // Smoothstep over the ±1-pixel anti-aliasing band.
                let t = (px_dist + 1.0) * 0.5;         // remap  −1…+1 → 0…1
                let t = t * t * (3.0 - 2.0 * t);        // cubic smoothstep
                (t * 255.0).round() as u8
            };

            // Only alpha is used downstream; RGB channels are zero.
            data[(y * w + x) * 4 + 3] = alpha;
        }
    }

    RasterBuffer { width: size, height: size, data }
}