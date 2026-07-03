use anyhow::Result;
use std::path::Path;

use crate::config::Config;

// ── .desktop file ─────────────────────────────────────────────────────────────

/// Write a freedesktop Desktop Entry file to `dest`.
///
/// Reference: <https://specifications.freedesktop.org/desktop-entry-spec/latest/>
pub fn write_desktop_file(config: &Config, dest: &Path) -> Result<()> {
    let content = build_desktop(config);
    write_text(dest, &content)
}

fn build_desktop(config: &Config) -> String {
    let app = &config.app;
    let mut s = String::new();

    line(&mut s, "[Desktop Entry]");
    kv(&mut s, "Type", "Application");
    kv(&mut s, "Version", "1.0");
    kv(&mut s, "Name", &de_escape(&app.name));
    opt_kv(&mut s, "GenericName", app.generic_name.as_deref().map(de_escape).as_deref());
    opt_kv(&mut s, "Comment", app.comment.as_deref().map(de_escape).as_deref());
    kv(&mut s, "Exec", &de_escape(&app.exec));
    // Icon= takes the bare app ID — no path, no extension
    kv(&mut s, "Icon", &app.id);
    kv(&mut s, "Terminal", "false");

    if !app.categories.is_empty() {
        // Categories must end with a trailing semicolon per spec
        kv(&mut s, "Categories", &format!("{};", app.categories.join(";")));
    }
    if !app.keywords.is_empty() {
        kv(&mut s, "Keywords", &format!("{};", app.keywords.join(";")));
    }

    kv(&mut s, "StartupNotify", "true");
    opt_kv(&mut s, "StartupWMClass", app.startup_wm_class.as_deref());

    if !app.mime_types.is_empty() {
        kv(&mut s, "MimeType", &format!("{};", app.mime_types.join(";")));
    }

    s
}

// ── AppStream metainfo.xml ────────────────────────────────────────────────────

/// Write an AppStream MetaInfo XML file to `dest`.
///
/// Reference: <https://www.freedesktop.org/software/appstream/docs/>
pub fn write_metainfo(config: &Config, dest: &Path) -> Result<()> {
    let content = build_metainfo(config);
    write_text(dest, &content)
}

fn build_metainfo(config: &Config) -> String {
    let app = &config.app;
    let mut s = String::new();

    line(&mut s, r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    line(&mut s, r#"<component type="desktop-application">"#);

    xml_elem(&mut s, 1, "id", &app.id);
    xml_elem(&mut s, 1, "name", &app.name);

    if let Some(summary) = &app.comment {
        xml_elem(&mut s, 1, "summary", summary);
    }

    if let Some(ml) = &app.metadata_license {
        xml_elem(&mut s, 1, "metadata_license", ml);
    }
    if let Some(pl) = &app.license {
        xml_elem(&mut s, 1, "project_license", pl);
    }

    // Link to the .desktop file
    s.push_str(&format!(
        "  <launchable type=\"desktop-id\">{}.desktop</launchable>\n",
        xml_esc(&app.id)
    ));

    // Stock icon name (matches hicolor filename stem)
    s.push_str(&format!(
        "  <icon type=\"stock\">{}</icon>\n",
        xml_esc(&app.id)
    ));

    if let Some(dev) = &app.developer_name {
        xml_elem(&mut s, 1, "developer_name", dev);
    }

    if let Some(url) = &app.homepage_url {
        s.push_str(&format!(
            "  <url type=\"homepage\">{}</url>\n",
            xml_esc(url)
        ));
    }

    // Description — wrapped in <p> tags
    if let Some(desc) = &app.description {
        line(&mut s, "  <description>");
        s.push_str(&format!("    <p>{}</p>\n", xml_esc(desc)));
        line(&mut s, "  </description>");
    }

    // MIME-type provisions
    if !app.mime_types.is_empty() {
        line(&mut s, "  <provides>");
        for mt in &app.mime_types {
            xml_elem(&mut s, 2, "mediatype", mt);
        }
        line(&mut s, "  </provides>");
    }

    // Categories
    if !app.categories.is_empty() {
        line(&mut s, "  <categories>");
        for cat in &app.categories {
            xml_elem(&mut s, 2, "category", cat);
        }
        line(&mut s, "  </categories>");
    }

    // Keywords
    if !app.keywords.is_empty() {
        line(&mut s, "  <keywords>");
        for kw in &app.keywords {
            xml_elem(&mut s, 2, "keyword", kw);
        }
        line(&mut s, "  </keywords>");
    }

    // OARS content rating — empty element signals "no objectionable content"
    line(&mut s, r#"  <content_rating type="oars-1.1" />"#);

    line(&mut s, "</component>");
    s
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn write_text(dest: &Path, content: &str) -> Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(dest, content)
        .map_err(|e| anyhow::anyhow!("cannot write '{}': {}", dest.display(), e))
}

fn line(s: &mut String, text: &str) {
    s.push_str(text);
    s.push('\n');
}

fn kv(s: &mut String, key: &str, value: &str) {
    s.push_str(key);
    s.push('=');
    s.push_str(value);
    s.push('\n');
}

fn opt_kv(s: &mut String, key: &str, value: Option<&str>) {
    if let Some(v) = value {
        kv(s, key, v);
    }
}

fn xml_elem(s: &mut String, depth: usize, tag: &str, content: &str) {
    let indent = "  ".repeat(depth);
    s.push_str(&format!("{}<{}>{}</{}>\n", indent, tag, xml_esc(content), tag));
}

/// Escape per the Desktop Entry Specification (backslash, newline, tab, CR).
fn de_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('\n', "\\n")
     .replace('\r', "\\r")
     .replace('\t', "\\t")
}

/// Escape the five XML predefined entities.
fn xml_esc(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&apos;")
}