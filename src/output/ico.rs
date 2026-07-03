use anyhow::Result;
use ico::{IconDir, IconDirEntry, IconImage, ResourceType};
use std::path::Path;

use crate::render::RasterBuffer;

/// Windows ICO sizes per Microsoft's guidelines.
/// The `ico` crate automatically stores 256px as PNG and smaller sizes as BMP.
pub const WINDOWS_SIZES: &[u32] = &[16, 24, 32, 48, 64, 128, 256];

/// Write a multi-size Windows .ico to `dest`.
/// `buffers` should be `(size, buffer)` pairs at each size in `WINDOWS_SIZES`.
/// Buffers must contain straight (non-premultiplied) RGBA — call `to_straight_alpha()` first,
/// which `write_ico` does internally.
pub fn write_ico(buffers: &[(u32, RasterBuffer)], dest: &Path) -> Result<()> {
    let mut dir = IconDir::new(ResourceType::Icon);

    for (size, buf) in buffers {
        let straight = buf.to_straight_alpha();
        let image = IconImage::from_rgba_data(buf.width, buf.height, straight);
        let entry = IconDirEntry::encode(&image)
            .map_err(|e| anyhow::anyhow!("ICO encode error at {}px: {}", size, e))?;
        dir.add_entry(entry);
    }

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = std::fs::File::create(dest)
        .map_err(|e| anyhow::anyhow!("cannot create '{}': {}", dest.display(), e))?;
    dir.write(file)
        .map_err(|e| anyhow::anyhow!("ICO write error: {}", e))
}