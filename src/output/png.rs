use anyhow::Result;
use image::{ExtendedColorType, ImageEncoder, codecs::png::PngEncoder};
use std::io::BufWriter;
use std::path::Path;

use crate::render::RasterBuffer;

/// Write a single straight-alpha RGBA PNG to `dest`.
/// Parent directories are created automatically.
pub fn write_png(buf: &RasterBuffer, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = std::fs::File::create(dest)
        .with_context(|| format!("cannot create '{}'", dest.display()))?;
    let encoder = PngEncoder::new(BufWriter::new(file));
    let straight = buf.to_straight_alpha();
    encoder
        .write_image(&straight, buf.width, buf.height, ExtendedColorType::Rgba8)
        .map_err(|e| anyhow::anyhow!("PNG encode error writing '{}': {}", dest.display(), e))
}

/// Emit the full freedesktop hicolor directory tree under `hicolor_dir`:
///
/// ```text
/// hicolor/
///   {size}x{size}/apps/{app_id}.png   for each size in `buffers`
///   scalable/apps/{app_id}.svg        (copy of master SVG)
///   symbolic/apps/{app_id}-symbolic.svg (if symbolic_svg is Some)
/// ```
pub fn write_hicolor_tree(
    buffers: &[(u32, RasterBuffer)],
    scalable_svg: &Path,
    symbolic_svg: Option<&Path>,
    app_id: &str,
    hicolor_dir: &Path,
) -> Result<()> {
    for (size, buf) in buffers {
        let dest = hicolor_dir
            .join(format!("{}x{}", size, size))
            .join("apps")
            .join(format!("{}.png", app_id));
        write_png(buf, &dest)?;
    }

    let scalable_dest = hicolor_dir
        .join("scalable")
        .join("apps")
        .join(format!("{}.svg", app_id));
    std::fs::create_dir_all(scalable_dest.parent().unwrap())?;
    std::fs::copy(scalable_svg, &scalable_dest)
        .map_err(|e| anyhow::anyhow!("cannot copy scalable SVG: {}", e))?;

    if let Some(sym) = symbolic_svg {
        let sym_dest = hicolor_dir
            .join("symbolic")
            .join("apps")
            .join(format!("{}-symbolic.svg", app_id));
        std::fs::create_dir_all(sym_dest.parent().unwrap())?;
        std::fs::copy(sym, &sym_dest)
            .map_err(|e| anyhow::anyhow!("cannot copy symbolic SVG: {}", e))?;
    }

    Ok(())
}

use anyhow::Context;