use anyhow::{bail, Result};
use std::path::Path;

use crate::config::Config;

/// Emit an AppImage asset tree under `out_dir/appimage/{AppName}.AppDir/`.
///
/// AppDir layout produced:
/// ```text
/// {AppName}.AppDir/
///   {app_id}.png                ← 256×256 top-level icon (Icon= basename + .png)
///   .DirIcon                    ← symlink → {app_id}.png
///   {app_id}.desktop            ← top-level desktop file (used by appimagetool)
///   usr/
///     share/
///       applications/{app_id}.desktop
///       metainfo/{app_id}.metainfo.xml
///       icons/hicolor/
///         128x128/apps/{app_id}.png
///         256x256/apps/{app_id}.png
///         scalable/apps/{app_id}.svg
/// ```
///
/// `AppRun` and the application binary are not generated — add those separately.
/// `linux_dir` must be the `dist/linux/` directory produced by the Linux phase.
pub fn write_appimage_bundle(config: &Config, linux_dir: &Path, out_dir: &Path) -> Result<()> {
    let app_id = &config.app.id;
    let app_name = config.app.name.replace(' ', "_");
    let hicolor_dir = linux_dir.join("hicolor");

    if !hicolor_dir.exists() {
        bail!(
            "AppImage packaging requires the Linux hicolor tree — enable [platforms] linux = true"
        );
    }

    let appdir = out_dir.join("appimage").join(format!("{}.AppDir", app_name));
    std::fs::create_dir_all(&appdir)?;

    // ── Top-level icon (Icon= basename + .png) ─────────────────────────────
    let icon_name = format!("{}.png", app_id);
    let icon_src = hicolor_dir
        .join("256x256")
        .join("apps")
        .join(&icon_name);
    if !icon_src.exists() {
        bail!("256×256 PNG not found at {}", icon_src.display());
    }
    std::fs::copy(&icon_src, appdir.join(&icon_name))?;

    // ── .DirIcon symlink → {app_id}.png ───────────────────────────────────
    let diricon = appdir.join(".DirIcon");
    if diricon.exists() || diricon.is_symlink() {
        std::fs::remove_file(&diricon)?;
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(&icon_name, &diricon)?;
    #[cfg(not(unix))]
    std::fs::copy(&icon_src, &diricon)?;  // fallback: plain copy on non-Unix builds

    // ── Top-level .desktop (required by appimagetool) ─────────────────────
    let desktop_src = linux_dir.join(format!("{}.desktop", app_id));
    if !desktop_src.exists() {
        bail!(".desktop not found at {}", desktop_src.display());
    }
    std::fs::copy(&desktop_src, appdir.join(format!("{}.desktop", app_id)))?;

    // ── Inner usr/share tree ───────────────────────────────────────────────
    let share = appdir.join("usr").join("share");

    // applications/ and metainfo/
    let apps_dir = share.join("applications");
    std::fs::create_dir_all(&apps_dir)?;
    std::fs::copy(&desktop_src, apps_dir.join(format!("{}.desktop", app_id)))?;

    let metainfo_src = linux_dir.join(format!("{}.metainfo.xml", app_id));
    if metainfo_src.exists() {
        let metainfo_dir = share.join("metainfo");
        std::fs::create_dir_all(&metainfo_dir)?;
        std::fs::copy(&metainfo_src, metainfo_dir.join(format!("{}.metainfo.xml", app_id)))?;
    }

    // Hicolor subset: 128×128, 256×256, scalable
    copy_hicolor_subset(&hicolor_dir, &share.join("icons").join("hicolor"), app_id)?;

    Ok(())
}

fn copy_hicolor_subset(hicolor_src: &Path, hicolor_dst: &Path, app_id: &str) -> Result<()> {
    for size in &[128u32, 256] {
        let dir_name = format!("{}x{}", size, size);
        let src = hicolor_src.join(&dir_name).join("apps").join(format!("{}.png", app_id));
        if src.exists() {
            let dst = hicolor_dst.join(&dir_name).join("apps").join(format!("{}.png", app_id));
            std::fs::create_dir_all(dst.parent().unwrap())?;
            std::fs::copy(&src, &dst)?;
        }
    }

    let scalable_src = hicolor_src
        .join("scalable")
        .join("apps")
        .join(format!("{}.svg", app_id));
    if scalable_src.exists() {
        let scalable_dst = hicolor_dst
            .join("scalable")
            .join("apps")
            .join(format!("{}.svg", app_id));
        std::fs::create_dir_all(scalable_dst.parent().unwrap())?;
        std::fs::copy(&scalable_src, &scalable_dst)?;
    }

    Ok(())
}