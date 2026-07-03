use clap::Parser;

/// Generate platform icon bundles from a single SVG.
///
/// Two ways to drive it, and they compose (flags override the config file):
///
///   • Config mode (IDE / toolchain): keep an icons.toml in your project and run
///     `iconsmaker` — assets land in the configured output dir (default dist/).
///
///   • Flag mode (web / CI backend): pass everything on the command line, e.g.
///     `iconsmaker -i logo.svg --all --id com.example.App --name "App" --exec app`.
///
/// Which fields are required depends on the platforms you select. Missing ones are
/// reported together, each naming the flag and the config key that would satisfy it.
#[derive(Parser, Debug)]
#[command(name = "iconsmaker", version, about, long_about = None)]
pub struct Cli {
    // ── Config ──────────────────────────────────────────────────────────────
    /// Path to an icons.toml config. If omitted, ./icons.toml is used when present.
    #[arg(short, long)]
    pub config: Option<String>,

    // ── Input / output ──────────────────────────────────────────────────────
    /// Master SVG to render (overrides [input] svg). May also be given positionally.
    #[arg(short = 'i', long = "input", value_name = "SVG")]
    pub input: Option<String>,

    /// Master SVG given as a positional argument (alternative to --input).
    #[arg(value_name = "SVG")]
    pub input_positional: Option<String>,

    /// Output directory root (overrides [output] dir; default: dist).
    #[arg(short = 'o', long = "output", value_name = "DIR")]
    pub output: Option<String>,

    /// Monochrome symbolic SVG for Linux trays (overrides [input] symbolic_svg).
    #[arg(long, value_name = "SVG")]
    pub symbolic: Option<String>,

    // ── Identity ([app]) ────────────────────────────────────────────────────
    /// Reverse-DNS application id, e.g. com.example.MyApp ([app] id).
    #[arg(long)]
    pub id: Option<String>,

    /// Human-readable application name ([app] name).
    #[arg(long)]
    pub name: Option<String>,

    /// .desktop Exec= launch command, e.g. "myapp %F" ([app] exec).
    #[arg(long)]
    pub exec: Option<String>,

    /// GenericName=, e.g. "Audio Editor" ([app] generic_name).
    #[arg(long)]
    pub generic_name: Option<String>,

    /// One-line summary ([app] comment).
    #[arg(long)]
    pub comment: Option<String>,

    /// Longer store-listing description ([app] description).
    #[arg(long)]
    pub description: Option<String>,

    /// freedesktop menu categories, comma-separated ([app] categories).
    #[arg(long, value_delimiter = ',')]
    pub categories: Option<Vec<String>>,

    /// Search keywords, comma-separated ([app] keywords).
    #[arg(long, value_delimiter = ',')]
    pub keywords: Option<Vec<String>>,

    /// StartupWMClass= — must match the window WM_CLASS ([app] startup_wm_class).
    #[arg(long)]
    pub startup_wm_class: Option<String>,

    /// MIME types, comma-separated ([app] mime_types).
    #[arg(long, value_delimiter = ',')]
    pub mime_types: Option<Vec<String>>,

    /// App SPDX license, e.g. GPL-3.0-or-later ([app] license).
    #[arg(long)]
    pub license: Option<String>,

    /// Metadata SPDX license, e.g. CC0-1.0 ([app] metadata_license).
    #[arg(long)]
    pub metadata_license: Option<String>,

    /// Project homepage URL ([app] homepage_url).
    #[arg(long)]
    pub homepage_url: Option<String>,

    /// Developer or studio name ([app] developer_name).
    #[arg(long)]
    pub developer_name: Option<String>,

    // ── Platform selection ──────────────────────────────────────────────────
    /// Generate macOS assets (.icns).
    #[arg(long)]
    pub macos: bool,

    /// Generate Windows assets (.ico).
    #[arg(long)]
    pub windows: bool,

    /// Generate Linux assets (hicolor tree, .desktop, metainfo.xml).
    #[arg(long)]
    pub linux: bool,

    /// Generate all three OS targets (macOS, Windows, Linux). Not packaging.
    #[arg(long)]
    pub all: bool,

    // ── Packaging ([packaging]) ─────────────────────────────────────────────
    /// Snap bundle scaffolding (requires --linux).
    #[arg(long)]
    pub snap: bool,

    /// Flatpak bundle scaffolding (requires --linux).
    #[arg(long)]
    pub flatpak: bool,

    /// AppImage bundle scaffolding (requires --linux).
    #[arg(long)]
    pub appimage: bool,

    // ── Effects ([effects]) ─────────────────────────────────────────────────
    /// Squircle superellipse power, >= 2.0 ([effects] squircle_power).
    #[arg(long)]
    pub squircle_power: Option<f32>,

    /// Enable macOS depth bevel shading ([effects] squircle_depth = true).
    #[arg(long = "squircle-depth", overrides_with = "no_squircle_depth")]
    pub squircle_depth: bool,

    /// Disable macOS depth bevel shading ([effects] squircle_depth = false).
    #[arg(long = "no-squircle-depth")]
    pub no_squircle_depth: bool,

    /// Depth lighting intensity, 0.0–1.0 ([effects] gloss_strength).
    #[arg(long)]
    pub gloss_strength: Option<f32>,

    /// Depth lighting blur, 0.0–1.0 ([effects] depth_blur).
    #[arg(long)]
    pub depth_blur: Option<f32>,

    /// Linux hicolor PNG sizes, comma-separated ([linux] hicolor_sizes).
    #[arg(long, value_delimiter = ',')]
    pub hicolor_sizes: Option<Vec<u32>>,

    // ── Modifiers ───────────────────────────────────────────────────────────
    /// Treat recommended-field warnings as hard errors.
    #[arg(long)]
    pub strict: bool,

    /// Print per-size render times and full output paths.
    #[arg(short, long)]
    pub verbose: bool,

    /// Write the iconsmaker(1) man page in groff format to stdout and exit.
    #[arg(long, hide = true)]
    pub print_man_page: bool,
}

impl Cli {
    /// Resolve the input SVG from either --input or the positional argument.
    /// Errors if both are given (ambiguous), returns None if neither (falls back
    /// to config).
    pub fn resolve_input(&self) -> anyhow::Result<Option<String>> {
        match (&self.input, &self.input_positional) {
            (Some(_), Some(_)) => anyhow::bail!(
                "the input SVG was given both positionally and with --input; pass it only once"
            ),
            (Some(s), None) | (None, Some(s)) => Ok(Some(s.clone())),
            (None, None) => Ok(None),
        }
    }

    /// Tri-state override for the depth bevel: Some(true)/Some(false) if a depth
    /// flag was passed, None if neither (falls back to config).
    pub fn depth_override(&self) -> Option<bool> {
        if self.squircle_depth {
            Some(true)
        } else if self.no_squircle_depth {
            Some(false)
        } else {
            None
        }
    }

    /// True if any platform or packaging flag was passed, meaning the CLI (not the
    /// config file) controls which targets are generated.
    pub fn controls_selection(&self) -> bool {
        self.all
            || self.macos
            || self.windows
            || self.linux
            || self.snap
            || self.flatpak
            || self.appimage
    }
}