use super::squircle::squircle_mask;
use super::RasterBuffer;

// ── Sampling constants ────────────────────────────────────────────────────────

const DIRS: [(i32, i32); 8] = [
    ( 1,  0), (-1,  0), ( 0,  1), ( 0, -1),
    ( 1,  1), (-1,  1), ( 1, -1), (-1, -1),
];
const SCALE: [f32; 8] = [1.0, 1.0, 1.0, 1.0, 1.414, 1.414, 1.414, 1.414];
const DIR_VEC: [(f32, f32); 8] = [
    ( 1.000,  0.000), (-1.000,  0.000),
    ( 0.000,  1.000), ( 0.000, -1.000),
    ( 0.707,  0.707), (-0.707,  0.707),
    ( 0.707, -0.707), (-0.707, -0.707),
];

// ── Public API ────────────────────────────────────────────────────────────────

/// Apply SDF-based bevel depth shading to `buf`.
///
/// Generates the squircle mask analytically from the superellipse formula
/// (no external file), then computes highlight and shadow float maps from the
/// boundary, Gaussian-blurs those maps, and composites the result onto the icon.
///
/// Because only the lighting maps are blurred — never the pixel data itself —
/// the icon's sharp foreground content stays unaffected.
///
/// `n`          — superellipse power matching the squircle mask (typically 5.0)
/// `strength`   — overall lighting intensity, 0.0–1.0
/// `depth_blur` — Gaussian sigma as a fraction of the bevel band; 0 = off
///
/// **Call BEFORE `apply_squircle_mask`** so edge highlights are clipped cleanly.
pub fn apply_squircle_depth(
    buf: &mut RasterBuffer,
    n: f32,
    strength: f32,
    depth_blur: f32,
) {
    let mask = squircle_mask(buf.width, n);
    let w    = buf.width  as usize;
    let h    = buf.height as usize;
    let s    = strength.clamp(0.0, 1.0);

    let max_dist = (buf.width.min(buf.height) as f32 * 0.07).max(4.0);
    let (lx, ly, lz) = normalise3(-0.35, -0.75, 0.50);

    // ── Pass 1: build bevel lighting maps ─────────────────────────────────────
    let mut h_map = vec![0.0f32; w * h];
    let mut s_map = vec![0.0f32; w * h];

    for y in 0..h {
        for x in 0..w {
            // Inside squircle = mask alpha ≈ 0; skip outside pixels.
            if mask.data[(y * w + x) * 4 + 3] > 192 { continue; }

            let (dist, (ox, oy)) = nearest_edge(x, y, w, h, &mask.data, max_dist);
            let bevel = (1.0 - dist / max_dist).clamp(0.0, 1.0).powi(2);
            if bevel < 0.001 { continue; }

            let tilt   = bevel * 0.90;
            let nz_raw = (1.0 - tilt).max(0.10);
            let nlen   = (tilt * tilt + nz_raw * nz_raw).sqrt();
            let (nx, ny_n, nz_n) = (ox * tilt / nlen, oy * tilt / nlen, nz_raw / nlen);
            let dot = nx * lx + ny_n * ly + nz_n * lz;

            h_map[y * w + x] = dot.max(0.0) * bevel * s;
            s_map[y * w + x] = (-dot).max(0.0) * bevel * s * 0.60;
        }
    }

    // ── Optional Gaussian blur on the bevel maps ───────────────────────────────
    let sigma = max_dist * depth_blur.clamp(0.0, 1.0);
    if sigma >= 0.5 {
        gaussian_blur_f32(&mut h_map, w, h, sigma);
        gaussian_blur_f32(&mut s_map, w, h, sigma);
    }

    // ── Pass 2: apply maps + ambient gradient to icon pixels ───────────────────
    for y in 0..h {
        let ny_norm = y as f32 / h as f32;
        let ambient = (0.5 - ny_norm) * s * 0.10;

        for x in 0..w {
            if mask.data[(y * w + x) * 4 + 3] > 192 { continue; }

            let highlight = h_map[y * w + x] + ambient.max(0.0);
            let shadow    = s_map[y * w + x] + (-ambient).max(0.0);

            let idx = (y * w + x) * 4;
            for c in 0..3 {
                let v = buf.data[idx + c] as f32;
                let v = v + (255.0 - v) * highlight;
                let v = v * (1.0 - shadow);
                buf.data[idx + c] = v.clamp(0.0, 255.0) as u8;
            }
        }
    }
}

// ── Internal: edge sampling ───────────────────────────────────────────────────

fn nearest_edge(
    x: usize, y: usize,
    w: usize, h: usize,
    mask: &[u8],
    max_dist: f32,
) -> (f32, (f32, f32)) {
    let mut min_dist = max_dist;
    let mut outward  = (0.0f32, -1.0f32);

    for i in 0..8 {
        let (dx, dy) = DIRS[i];
        let scale    = SCALE[i];
        let max_step = (max_dist / scale).ceil() as i32;

        let mut d    = 1i32;
        let mut last = 0i32;

        loop {
            let nx = x as i32 + dx * d;
            let ny = y as i32 + dy * d;

            // alpha > 128 → outside the squircle boundary
            let outside = nx < 0 || ny < 0
                || nx >= w as i32 || ny >= h as i32
                || mask[(ny as usize * w + nx as usize) * 4 + 3] > 128;

            if outside {
                let dist = (last as f32 + d as f32) * 0.5 * scale;
                if dist < min_dist {
                    min_dist = dist;
                    outward  = DIR_VEC[i];
                }
                break;
            }

            last = d;
            if d >= max_step { break; }
            d = (d * 2).min(max_step);
        }
    }

    (min_dist, outward)
}

// ── Internal: separable Gaussian blur on f32 maps ────────────────────────────

fn gaussian_blur_f32(map: &mut [f32], w: usize, h: usize, sigma: f32) {
    let radius = (3.0 * sigma).ceil() as usize;
    let size   = 2 * radius + 1;

    let mut kernel = vec![0.0f32; size];
    let mut sum    = 0.0f32;
    for i in 0..size {
        let x   = (i as i32 - radius as i32) as f32;
        kernel[i] = (-x * x / (2.0 * sigma * sigma)).exp();
        sum += kernel[i];
    }
    for k in &mut kernel { *k /= sum; }

    let mut tmp = vec![0.0f32; w * h];

    for y in 0..h {
        for x in 0..w {
            let mut acc = 0.0f32;
            for (ki, &kv) in kernel.iter().enumerate() {
                let sx = (x as i32 + ki as i32 - radius as i32).clamp(0, w as i32 - 1) as usize;
                acc += map[y * w + sx] * kv;
            }
            tmp[y * w + x] = acc;
        }
    }

    for y in 0..h {
        for x in 0..w {
            let mut acc = 0.0f32;
            for (ki, &kv) in kernel.iter().enumerate() {
                let sy = (y as i32 + ki as i32 - radius as i32).clamp(0, h as i32 - 1) as usize;
                acc += tmp[sy * w + x] * kv;
            }
            map[y * w + x] = acc;
        }
    }
}

// ── Utility ───────────────────────────────────────────────────────────────────

#[inline]
fn normalise3(x: f32, y: f32, z: f32) -> (f32, f32, f32) {
    let len = (x * x + y * y + z * z).sqrt();
    (x / len, y / len, z / len)
}