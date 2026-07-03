use anyhow::Result;
use icns::{IconFamily, IconType, Image, PixelFormat};
use std::path::Path;

use crate::render::RasterBuffer;

/// macOS ICNS pixel sizes.
/// 1024 is stored as the `ic10` (512@2x) element — the largest Retina variant.
pub const MACOS_SIZES: &[u32] = &[16, 32, 64, 128, 256, 512, 1024];

/// Map a pixel size to the corresponding modern RGBA ICNS element type.
/// All types here use the PNG-compressed RGBA32 encoding (macOS 10.7+).
fn icns_type(size: u32) -> Option<IconType> {
    match size {
        16   => Some(IconType::RGBA32_16x16),
        32   => Some(IconType::RGBA32_32x32),
        64   => Some(IconType::RGBA32_64x64),
        128  => Some(IconType::RGBA32_128x128),
        256  => Some(IconType::RGBA32_256x256),
        512  => Some(IconType::RGBA32_512x512),
        1024 => Some(IconType::RGBA32_512x512_2x),
        _    => None,
    }
}

/// Write a macOS .icns bundle to `dest`.
/// `buffers` must be squircle-masked RGBA pairs at each size in `MACOS_SIZES`.
/// Premultiplied-to-straight conversion is handled internally.
pub fn write_icns(buffers: &[(u32, RasterBuffer)], dest: &Path) -> Result<()> {
    let mut family = IconFamily::new();

    for (size, buf) in buffers {
        let icon_type = icns_type(*size)
            .ok_or_else(|| anyhow::anyhow!("no ICNS type for size {}px", size))?;

        let straight = buf.to_straight_alpha();
        let image = Image::from_data(PixelFormat::RGBA, buf.width, buf.height, straight)
            .map_err(|e| anyhow::anyhow!("ICNS image error at {}px: {}", size, e))?;

        family
            .add_icon_with_type(&image, icon_type)
            .map_err(|e| anyhow::anyhow!("ICNS encode error at {}px: {}", size, e))?;
    }

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = std::fs::File::create(dest)
        .map_err(|e| anyhow::anyhow!("cannot create '{}': {}", dest.display(), e))?;
    family
        .write(file)
        .map_err(|e| anyhow::anyhow!("ICNS write error: {}", e))
}