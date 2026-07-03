use anyhow::{bail, Result};
use std::path::Path;

use crate::config::Config;

/// Emit a Snap store icon bundle under `out_dir/snap/`.
///
/// Output mirrors the layout expected in a snap source tree:
/// ```text
/// snap/
///   gui/
///     icon.png                 ← 256×256 PNG (< 256 KB required by Snap Store)
///     {app_id}.desktop         ← copy of the standard .desktop file
/// ```
///
/// Reference: `icon:` and `desktop:` keys in snapcraft.yaml.
/// `linux_dir` must be the `dist/linux/` directory produced by the Linux phase.
pub fn write_snap_bundle(config: &Config, linux_dir: &Path, out_dir: &Path) -> Result<()> {
    let app_id = &config.app.id;
    let hicolor_dir = linux_dir.join("hicolor");

    if !hicolor_dir.exists() {
        bail!(
            "Snap packaging requires the Linux hicolor tree — enable [platforms] linux = true"
        );
    }

    let snap_gui = out_dir.join("snap").join("snap").join("gui");
    std::fs::create_dir_all(&snap_gui)?;

    // Store icon: 256×256 PNG at snap/gui/icon.png
    let icon_src = hicolor_dir
        .join("256x256")
        .join("apps")
        .join(format!("{}.png", app_id));
    if !icon_src.exists() {
        bail!("256×256 PNG not found at {} — needed for Snap store icon", icon_src.display());
    }
    std::fs::copy(&icon_src, snap_gui.join("icon.png"))?;

    // Desktop file: snap/gui/{app_id}.desktop
    let desktop_src = linux_dir.join(format!("{}.desktop", app_id));
    if !desktop_src.exists() {
        bail!(
            ".desktop file not found at {} — run the Linux phase first",
            desktop_src.display()
        );
    }
    std::fs::copy(&desktop_src, snap_gui.join(format!("{}.desktop", app_id)))?;

    Ok(())
}