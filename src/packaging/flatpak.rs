use anyhow::{bail, Result};
use std::path::Path;

use crate::config::Config;
use super::copy_dir_all;

/// Emit a Flatpak asset tree under `out_dir/flatpak/`.
///
/// Output mirrors the install prefix inside a Flatpak sandbox:
/// ```text
/// flatpak/
///   app/
///     share/
///       applications/{app_id}.desktop
///       metainfo/{app_id}.metainfo.xml
///       icons/hicolor/           ← full copy of the hicolor tree
/// ```
///
/// In a Flatpak manifest the `install` commands point into this tree:
/// ```yaml
/// - install -Dm644 ... /app/share/icons/hicolor/256x256/apps/{app_id}.png
/// ```
///
/// `linux_dir` must be the `dist/linux/` directory produced by the Linux phase.
pub fn write_flatpak_bundle(config: &Config, linux_dir: &Path, out_dir: &Path) -> Result<()> {
    let app_id = &config.app.id;
    let hicolor_dir = linux_dir.join("hicolor");

    if !hicolor_dir.exists() {
        bail!(
            "Flatpak packaging requires the Linux hicolor tree — enable [platforms] linux = true"
        );
    }

    let share = out_dir.join("flatpak").join("app").join("share");

    // Full hicolor tree
    copy_dir_all(&hicolor_dir, &share.join("icons").join("hicolor"))?;

    // .desktop file
    let desktop_src = linux_dir.join(format!("{}.desktop", app_id));
    if !desktop_src.exists() {
        bail!(".desktop not found at {}", desktop_src.display());
    }
    let apps_dir = share.join("applications");
    std::fs::create_dir_all(&apps_dir)?;
    std::fs::copy(&desktop_src, apps_dir.join(format!("{}.desktop", app_id)))?;

    // metainfo.xml
    let metainfo_src = linux_dir.join(format!("{}.metainfo.xml", app_id));
    if !metainfo_src.exists() {
        bail!("metainfo.xml not found at {}", metainfo_src.display());
    }
    let metainfo_dir = share.join("metainfo");
    std::fs::create_dir_all(&metainfo_dir)?;
    std::fs::copy(&metainfo_src, metainfo_dir.join(format!("{}.metainfo.xml", app_id)))?;

    Ok(())
}