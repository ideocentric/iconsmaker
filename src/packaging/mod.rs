pub mod appimage;
pub mod flatpak;
pub mod snap;

/// Recursively copy `src` directory into `dst`, creating `dst` if needed.
pub(super) fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dst_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(&entry.path(), &dst_path).map_err(|e| {
                anyhow::anyhow!("copy {} → {}: {}", entry.path().display(), dst_path.display(), e)
            })?;
        }
    }
    Ok(())
}