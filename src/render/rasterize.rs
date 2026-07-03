use anyhow::{Context, Result};
use std::path::Path;
use tiny_skia::Transform;

use super::RasterBuffer;

/// Parse the SVG once, then render at every requested size.
pub fn rasterize_sizes(svg_path: &Path, sizes: &[u32]) -> Result<Vec<RasterBuffer>> {
    let data = std::fs::read(svg_path)
        .with_context(|| format!("cannot read SVG '{}'", svg_path.display()))?;
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_data(&data, &opt)
        .with_context(|| format!("cannot parse SVG '{}'", svg_path.display()))?;
    sizes.iter().map(|&s| render_tree(&tree, s)).collect()
}

/// Parse once and return `(size, buffer)` pairs — convenience wrapper over `rasterize_sizes`.
pub fn rasterize_as_pairs(svg_path: &Path, sizes: &[u32]) -> Result<Vec<(u32, RasterBuffer)>> {
    let buffers = rasterize_sizes(svg_path, sizes)?;
    Ok(sizes.iter().copied().zip(buffers).collect())
}

fn render_tree(tree: &usvg::Tree, size: u32) -> Result<RasterBuffer> {
    let mut pixmap = tiny_skia::Pixmap::new(size, size)
        .ok_or_else(|| anyhow::anyhow!("cannot allocate {}×{} pixmap", size, size))?;

    let svg_w = tree.size().width();
    let svg_h = tree.size().height();

    // Scale to fit (preserving aspect ratio), then centre in the target square.
    let scale = (size as f32 / svg_w).min(size as f32 / svg_h);
    let tx = (size as f32 - svg_w * scale) / 2.0;
    let ty = (size as f32 - svg_h * scale) / 2.0;

    // pre_scale means scale is applied before translate:
    // effective transform = T(tx,ty) × S(scale) → scale first, then shift.
    let transform = Transform::from_translate(tx, ty).pre_scale(scale, scale);

    resvg::render(tree, transform, &mut pixmap.as_mut());

    Ok(RasterBuffer {
        width: size,
        height: size,
        data: pixmap.take(),
    })
}