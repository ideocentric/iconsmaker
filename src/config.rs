use serde::Deserialize;

// ============================================================================
// RawConfig — the TOML deserialization + merge layer.
//
// Every identity field is Option so that "absent in the file" is distinguishable
// from "present". CLI flags are layered on top (flag.or(config)) in validate.rs,
// and the merged result is validated against the selected platforms before being
// lowered into the concrete `Config` consumed by the pipeline.
// ============================================================================

#[derive(Debug, Default, Deserialize)]
pub struct RawConfig {
    #[serde(default)]
    pub app: RawApp,
    #[serde(default)]
    pub input: RawInput,
    #[serde(default)]
    pub output: OutputConfig,
    #[serde(default)]
    pub platforms: PlatformConfig,
    #[serde(default)]
    pub linux: LinuxConfig,
    #[serde(default)]
    pub effects: EffectsConfig,
    #[serde(default)]
    pub packaging: PackagingConfig,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawApp {
    pub id: Option<String>,
    pub name: Option<String>,
    pub generic_name: Option<String>,
    pub comment: Option<String>,
    pub exec: Option<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    pub startup_wm_class: Option<String>,
    #[serde(default)]
    pub mime_types: Vec<String>,
    pub license: Option<String>,
    pub metadata_license: Option<String>,
    pub description: Option<String>,
    pub homepage_url: Option<String>,
    pub developer_name: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawInput {
    pub svg: Option<String>,
    pub symbolic_svg: Option<String>,
}

impl RawConfig {
    /// Parse an icons.toml file. Missing files are reported with a targeted hint;
    /// presence/requiredness of individual fields is checked later by the grid.
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow::anyhow!(
                    "config file not found: '{}'\n  \
                     Run iconsmaker from your project directory, \
                     or pass --config path/to/icons.toml",
                    path
                )
            } else {
                anyhow::anyhow!("cannot read '{}': {}", path, e)
            }
        })?;
        toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("cannot parse '{}': {}", path, e))
    }
}

// ============================================================================
// Config — the resolved, validated configuration handed to the pipeline.
//
// Produced by validate::resolve. Every field here is guaranteed to satisfy the
// requirements of the *selected* platforms; fields that a selected platform does
// not consume may be empty (they are never read because each pipeline block is
// platform-guarded).
// ============================================================================

#[derive(Debug)]
pub struct Config {
    pub app: AppConfig,
    pub input: InputConfig,
    pub output: OutputConfig,
    pub platforms: PlatformConfig,
    pub linux: LinuxConfig,
    pub effects: EffectsConfig,
    pub packaging: PackagingConfig,
}

#[derive(Debug)]
pub struct AppConfig {
    /// Reverse-DNS application identifier (e.g. com.example.MyApp).
    /// Used as the filename stem for all assets and as the Flatpak/Snap app ID.
    pub id: String,
    pub name: String,
    pub generic_name: Option<String>,
    pub comment: Option<String>,
    /// Shell command for the .desktop Exec= field (e.g. "myapp %F")
    pub exec: String,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
    /// Must match the window's WM_CLASS for correct dock/taskbar grouping on Linux
    pub startup_wm_class: Option<String>,
    pub mime_types: Vec<String>,
    pub license: Option<String>,
    pub metadata_license: Option<String>,
    /// One-paragraph plain-text description for AppStream store listings
    pub description: Option<String>,
    /// Project homepage — emitted as <url type="homepage"> in metainfo.xml
    pub homepage_url: Option<String>,
    /// Human-readable developer or studio name
    pub developer_name: Option<String>,
}

#[derive(Debug)]
pub struct InputConfig {
    /// Path to the 1024×1024 master SVG
    pub svg: String,
    /// Monochrome symbolic SVG for Linux trays/panels (optional)
    pub symbolic_svg: Option<String>,
}

// ── Shared sections (identical shape in RawConfig and Config) ────────────────

#[derive(Debug, Deserialize)]
pub struct OutputConfig {
    #[serde(default = "default_dist")]
    pub dir: String,
}

#[derive(Debug, Deserialize)]
pub struct PlatformConfig {
    #[serde(default = "default_true")]
    pub macos: bool,
    #[serde(default = "default_true")]
    pub windows: bool,
    #[serde(default = "default_true")]
    pub linux: bool,
}

#[derive(Debug, Deserialize)]
pub struct LinuxConfig {
    #[serde(default = "default_hicolor_sizes")]
    pub hicolor_sizes: Vec<u32>,
}

#[derive(Debug, Deserialize)]
pub struct EffectsConfig {
    /// Superellipse power for the squircle shape applied to macOS icons.
    ///   n = 2 → circle, n = 4 → classic squircle, n = 5 → macOS-like (default)
    #[serde(default = "default_squircle_power")]
    pub squircle_power: f32,

    /// SDF bevel shading: rim highlight on the top edge, shadow on the bottom.
    /// Applied before squircle masking. Set false for a flat icon.
    #[serde(default)]
    pub squircle_depth: bool,

    /// Lighting intensity: 0.0 = invisible, 1.0 = maximum. Default 0.35.
    #[serde(default = "default_gloss_strength")]
    pub gloss_strength: f32,

    /// Gaussian blur on the lighting maps before compositing.
    /// Expressed as a fraction of the bevel band width.
    ///   0.00 = no blur (may show subtle staircase)
    ///   0.12 = default (smooth without losing edge definition)
    ///   0.30 = very soft
    #[serde(default = "default_depth_blur")]
    pub depth_blur: f32,
}

impl Default for EffectsConfig {
    fn default() -> Self {
        Self {
            squircle_power: default_squircle_power(),
            squircle_depth: false,
            gloss_strength: default_gloss_strength(),
            depth_blur: default_depth_blur(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct PackagingConfig {
    #[serde(default)]
    pub snap: bool,
    #[serde(default)]
    pub flatpak: bool,
    #[serde(default)]
    pub appimage: bool,
    #[serde(default)]
    pub deb: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self { dir: default_dist() }
    }
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self { macos: true, windows: true, linux: true }
    }
}

impl Default for LinuxConfig {
    fn default() -> Self {
        Self { hicolor_sizes: default_hicolor_sizes() }
    }
}

fn default_true() -> bool { true }
fn default_dist() -> String { "dist".to_string() }
fn default_hicolor_sizes() -> Vec<u32> { vec![16, 22, 24, 32, 48, 64, 128, 256, 512] }
fn default_squircle_power() -> f32 { 5.0 }
fn default_gloss_strength() -> f32 { 0.35 }
fn default_depth_blur()     -> f32 { 0.12 }