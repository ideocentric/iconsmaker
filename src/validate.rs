use std::path::Path;

use crate::cli::Cli;
use crate::config::{
    AppConfig, Config, EffectsConfig, InputConfig, LinuxConfig, OutputConfig, PackagingConfig,
    PlatformConfig, RawConfig,
};

// ============================================================================
// The validation grid.
//
// `resolve` merges CLI flags over the config file (flags win), works out which
// platforms are selected, then evaluates a requirement grid that depends on that
// selection. Missing HARD requirements block with an aggregated, fix-oriented
// message; missing SOFT recommendations warn (and, under --strict, become hard).
// ============================================================================

#[derive(Clone, Copy)]
pub struct Selection {
    pub macos: bool,
    pub windows: bool,
    pub linux: bool,
    pub snap: bool,
    pub flatpak: bool,
    pub appimage: bool,
}

/// All settings after merging CLI over config — the input to the grid. Kept as a
/// plain struct (no clap, no fs) so the grid can be unit-tested directly.
struct Merged {
    svg: Option<String>,
    svg_exists: bool,
    symbolic: Option<String>,

    id: Option<String>,
    name: Option<String>,
    exec: Option<String>,
    generic_name: Option<String>,
    comment: Option<String>,
    description: Option<String>,
    categories: Vec<String>,
    keywords: Vec<String>,
    startup_wm_class: Option<String>,
    mime_types: Vec<String>,
    license: Option<String>,
    metadata_license: Option<String>,
    homepage_url: Option<String>,
    developer_name: Option<String>,

    output_dir: String,
    hicolor_sizes: Vec<u32>,
    squircle_power: f32,
    squircle_depth: bool,
    gloss_strength: f32,
    depth_blur: f32,
    deb: bool,

    sel: Selection,
}

struct HardProblem {
    headline: String,
    detail: Vec<(&'static str, String)>,
}

impl HardProblem {
    fn new(headline: impl Into<String>, detail: Vec<(&'static str, String)>) -> Self {
        Self { headline: headline.into(), detail }
    }
}

struct SoftProblem {
    headline: String,
    hint: String,
}

struct Problems {
    hard: Vec<HardProblem>,
    soft: Vec<SoftProblem>,
}

// ── Public entry point ───────────────────────────────────────────────────────

/// Merge, resolve platform selection, run the grid, and lower into a concrete
/// `Config`. On hard failure returns an Err whose Display is the full formatted
/// error block (main prints it verbatim). Warnings are printed to stderr here.
pub fn resolve(cli: &Cli, raw: RawConfig, config_loaded: bool) -> anyhow::Result<Config> {
    // Selection precedence:
    //   1. any platform/packaging flag  → the CLI drives selection
    //   2. else a config was loaded     → its [platforms]/[packaging] drive it
    //   3. else (flags-only, none given) → nothing selected → "no platform" error
    let sel = if cli.controls_selection() {
        Selection {
            macos: cli.all || cli.macos,
            windows: cli.all || cli.windows,
            linux: cli.all || cli.linux,
            snap: cli.snap,
            flatpak: cli.flatpak,
            appimage: cli.appimage,
        }
    } else if config_loaded {
        Selection {
            macos: raw.platforms.macos,
            windows: raw.platforms.windows,
            linux: raw.platforms.linux,
            snap: raw.packaging.snap,
            flatpak: raw.packaging.flatpak,
            appimage: raw.packaging.appimage,
        }
    } else {
        Selection {
            macos: false,
            windows: false,
            linux: false,
            snap: false,
            flatpak: false,
            appimage: false,
        }
    };

    // Merge flags over config (flag.or(config); lists replace wholesale).
    let svg = cli.resolve_input()?.or(raw.input.svg);
    let svg_exists = svg.as_deref().is_some_and(|p| Path::new(p).exists());

    let m = Merged {
        svg,
        svg_exists,
        symbolic: cli.symbolic.clone().or(raw.input.symbolic_svg),
        id: cli.id.clone().or(raw.app.id),
        name: cli.name.clone().or(raw.app.name),
        exec: cli.exec.clone().or(raw.app.exec),
        generic_name: cli.generic_name.clone().or(raw.app.generic_name),
        comment: cli.comment.clone().or(raw.app.comment),
        description: cli.description.clone().or(raw.app.description),
        categories: cli.categories.clone().unwrap_or(raw.app.categories),
        keywords: cli.keywords.clone().unwrap_or(raw.app.keywords),
        startup_wm_class: cli.startup_wm_class.clone().or(raw.app.startup_wm_class),
        mime_types: cli.mime_types.clone().unwrap_or(raw.app.mime_types),
        license: cli.license.clone().or(raw.app.license),
        metadata_license: cli.metadata_license.clone().or(raw.app.metadata_license),
        homepage_url: cli.homepage_url.clone().or(raw.app.homepage_url),
        developer_name: cli.developer_name.clone().or(raw.app.developer_name),
        output_dir: cli.output.clone().unwrap_or(raw.output.dir),
        hicolor_sizes: cli.hicolor_sizes.clone().unwrap_or(raw.linux.hicolor_sizes),
        squircle_power: cli.squircle_power.unwrap_or(raw.effects.squircle_power),
        squircle_depth: cli.depth_override().unwrap_or(raw.effects.squircle_depth),
        gloss_strength: cli.gloss_strength.unwrap_or(raw.effects.gloss_strength),
        depth_blur: cli.depth_blur.unwrap_or(raw.effects.depth_blur),
        deb: raw.packaging.deb,
        sel,
    };

    let mut problems = evaluate(&m);

    // --strict: promote every soft warning to a hard error.
    if cli.strict {
        for sp in problems.soft.drain(..) {
            problems
                .hard
                .push(HardProblem::new(sp.headline, vec![line("fix", sp.hint)]));
        }
    }

    if !problems.hard.is_empty() {
        anyhow::bail!("{}", render_hard(&problems.hard));
    }
    if !problems.soft.is_empty() {
        eprint!("{}", render_soft(&problems.soft));
    }

    Ok(build_config(m))
}

// ── The grid (pure; unit-tested) ─────────────────────────────────────────────

fn evaluate(m: &Merged) -> Problems {
    let mut hard = Vec::new();
    let mut soft = Vec::new();
    let sel = m.sel;
    let any_os = sel.macos || sel.windows || sel.linux;

    // Global: at least one OS target.
    if !any_os {
        hard.push(HardProblem::new(
            "no platform selected",
            vec![
                line("because", "nothing would be generated"),
                line("provide", "--macos / --windows / --linux / --all"),
                line("or in config", "[platforms] macos = true"),
            ],
        ));
    }

    // Packaging requires Linux.
    let pkgs: Vec<&str> = [
        (sel.snap, "--snap"),
        (sel.flatpak, "--flatpak"),
        (sel.appimage, "--appimage"),
    ]
    .iter()
    .filter(|(on, _)| *on)
    .map(|(_, f)| *f)
    .collect();
    if !pkgs.is_empty() && !sel.linux {
        let verb = if pkgs.len() == 1 { "requires" } else { "require" };
        hard.push(HardProblem::new(
            format!("{} {verb} --linux", pkgs.join(", ")),
            vec![
                line(
                    "because",
                    "packaging bundles are built from the Linux hicolor tree + metadata",
                ),
                line("fix", "add --linux (or set [platforms] linux = true)"),
            ],
        ));
    }

    // Input SVG — needed whenever any OS is selected.
    if any_os {
        if !filled(&m.svg) {
            hard.push(HardProblem::new(
                "no input SVG",
                vec![
                    line("because", "there is nothing to render"),
                    line("provide", "--input path/to/master.svg  (or a positional SVG)"),
                    line("or in config", "[input] svg = \"icons/master.svg\""),
                ],
            ));
        } else if !m.svg_exists {
            hard.push(HardProblem::new(
                "input SVG not found",
                vec![
                    line("path", m.svg.clone().unwrap_or_default()),
                    line("fix", "check the path passed to --input / [input] svg"),
                ],
            ));
        }
    }

    // Required identity fields (HARD), gated by which selected targets need them.
    require(
        &mut hard,
        by(&sel, true, true, true),
        filled(&m.name),
        "application name",
        "names the .icns/.ico file, .desktop Name= and AppStream <name>",
        "--name \"My App\"",
        "[app] name = \"My App\"",
    );
    require(
        &mut hard,
        by(&sel, true, false, true),
        filled(&m.id),
        "application id (reverse-DNS)",
        "names every asset, the .desktop Icon= and AppStream <id>",
        "--id com.example.MyApp",
        "[app] id = \"com.example.MyApp\"",
    );
    require(
        &mut hard,
        by(&sel, false, false, true),
        filled(&m.exec),
        "exec command",
        "the .desktop Exec= launch command",
        "--exec \"myapp %F\"",
        "[app] exec = \"myapp %F\"",
    );

    // Recommended Linux metadata (SOFT) — omitting these fails validators.
    if sel.linux {
        recommend(&mut soft, filled(&m.comment), "comment (summary) missing",
            "--comment \"One-line summary\"", "[app] comment = \"…\"");
        recommend(&mut soft, filled(&m.description), "description missing",
            "--description \"…\"", "[app] description = \"…\"");
        recommend(&mut soft, !m.categories.is_empty(), "categories missing",
            "--categories Utility", "[app] categories = [\"Utility\"]");
        recommend(&mut soft, filled(&m.metadata_license), "metadata_license missing",
            "--metadata-license CC0-1.0", "[app] metadata_license = \"CC0-1.0\"");
        recommend(&mut soft, filled(&m.license), "license missing",
            "--license GPL-3.0-or-later", "[app] license = \"GPL-3.0-or-later\"");
        recommend(&mut soft, filled(&m.developer_name), "developer_name missing",
            "--developer-name \"Your Name\"", "[app] developer_name = \"Your Name\"");
    }

    // Format lints (SOFT).
    if let Some(id) = m.id.as_deref()
        && !id.trim().is_empty()
        && !looks_reverse_dns(id)
    {
        soft.push(SoftProblem {
            headline: format!("id \"{id}\" is not reverse-DNS"),
            hint: "use e.g. --id com.example.MyApp".to_string(),
        });
    }
    if sel.linux && !m.categories.is_empty() {
        let bad: Vec<&str> = m
            .categories
            .iter()
            .map(String::as_str)
            .filter(|c| !REGISTERED_CATEGORIES.contains(c))
            .collect();
        if !bad.is_empty() {
            soft.push(SoftProblem {
                headline: format!("unregistered categories: {}", bad.join(", ")),
                hint: "use values from the freedesktop menu spec, e.g. Utility, AudioVideo"
                    .to_string(),
            });
        }
    }

    // Effects ranges (HARD) — only relevant when macOS is selected.
    if sel.macos {
        if m.squircle_power < 2.0 {
            hard.push(HardProblem::new(
                "invalid squircle_power",
                vec![
                    line("value", m.squircle_power.to_string()),
                    line("allowed", ">= 2.0 (below 2 is non-convex)"),
                    line("fix", "--squircle-power 5.0"),
                ],
            ));
        }
        if !(0.0..=1.0).contains(&m.gloss_strength) {
            hard.push(HardProblem::new(
                "invalid gloss_strength",
                vec![
                    line("value", m.gloss_strength.to_string()),
                    line("allowed", "0.0 – 1.0"),
                    line("fix", "--gloss-strength 0.35"),
                ],
            ));
        }
        if !(0.0..=1.0).contains(&m.depth_blur) {
            hard.push(HardProblem::new(
                "invalid depth_blur",
                vec![
                    line("value", m.depth_blur.to_string()),
                    line("allowed", "0.0 – 1.0"),
                    line("fix", "--depth-blur 0.12"),
                ],
            ));
        }
    }

    Problems { hard, soft }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn filled(o: &Option<String>) -> bool {
    o.as_deref().is_some_and(|s| !s.trim().is_empty())
}

fn line(key: &'static str, value: impl Into<String>) -> (&'static str, String) {
    (key, value.into())
}

/// Which of the currently-selected targets require a given field.
fn by(sel: &Selection, macos: bool, windows: bool, linux: bool) -> Vec<&'static str> {
    let mut v = Vec::new();
    if macos && sel.macos {
        v.push("--macos");
    }
    if windows && sel.windows {
        v.push("--windows");
    }
    if linux && sel.linux {
        v.push("--linux");
    }
    v
}

#[allow(clippy::too_many_arguments)]
fn require(
    hard: &mut Vec<HardProblem>,
    required_by: Vec<&str>,
    present: bool,
    label: &str,
    because: &str,
    provide: &str,
    config: &str,
) {
    if required_by.is_empty() || present {
        return;
    }
    hard.push(HardProblem::new(
        label.to_string(),
        vec![
            line("required by", required_by.join(", ")),
            line("because", because.to_string()),
            line("provide", provide.to_string()),
            line("or in config", config.to_string()),
        ],
    ));
}

fn recommend(soft: &mut Vec<SoftProblem>, present: bool, headline: &str, flag: &str, config: &str) {
    if present {
        return;
    }
    soft.push(SoftProblem {
        headline: headline.to_string(),
        hint: format!("{flag}   |   {config}"),
    });
}

fn looks_reverse_dns(s: &str) -> bool {
    s.contains('.')
        && !s.chars().any(char::is_whitespace)
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
}

fn render_hard(hard: &[HardProblem]) -> String {
    let mut s = String::from("error: cannot generate the requested icons\n\n");
    for p in hard {
        s.push_str(&format!("  ✗ {}\n", p.headline));
        for (k, v) in &p.detail {
            s.push_str(&format!("      {k:<13}{v}\n"));
        }
        s.push('\n');
    }
    s.push_str("run `iconsmaker --help` for the full option list.");
    s
}

fn render_soft(soft: &[SoftProblem]) -> String {
    let mut s = format!("warning: {} recommended setting(s) to review\n", soft.len());
    for p in soft {
        s.push_str(&format!("  ⚠ {} — {}\n", p.headline, p.hint));
    }
    s.push_str("  (pass --strict to treat these as errors)\n");
    s
}

fn build_config(m: Merged) -> Config {
    Config {
        app: AppConfig {
            id: m.id.unwrap_or_default(),
            name: m.name.unwrap_or_default(),
            generic_name: m.generic_name,
            comment: m.comment,
            exec: m.exec.unwrap_or_default(),
            categories: m.categories,
            keywords: m.keywords,
            startup_wm_class: m.startup_wm_class,
            mime_types: m.mime_types,
            license: m.license,
            metadata_license: m.metadata_license,
            description: m.description,
            homepage_url: m.homepage_url,
            developer_name: m.developer_name,
        },
        input: InputConfig {
            svg: m.svg.unwrap_or_default(),
            symbolic_svg: m.symbolic,
        },
        output: OutputConfig { dir: m.output_dir },
        platforms: PlatformConfig {
            macos: m.sel.macos,
            windows: m.sel.windows,
            linux: m.sel.linux,
        },
        linux: LinuxConfig {
            hicolor_sizes: m.hicolor_sizes,
        },
        effects: EffectsConfig {
            squircle_power: m.squircle_power,
            squircle_depth: m.squircle_depth,
            gloss_strength: m.gloss_strength,
            depth_blur: m.depth_blur,
        },
        packaging: PackagingConfig {
            snap: m.sel.snap,
            flatpak: m.sel.flatpak,
            appimage: m.sel.appimage,
            deb: m.deb,
        },
    }
}

/// freedesktop menu-spec registered categories (main + additional). Used only to
/// lint `[app] categories`; unknown values warn but do not block.
const REGISTERED_CATEGORIES: &[&str] = &[
    // Main
    "AudioVideo", "Audio", "Video", "Development", "Education", "Game", "Graphics",
    "Network", "Office", "Science", "Settings", "System", "Utility",
    // Additional
    "Building", "Debugger", "IDE", "GUIDesigner", "Profiling", "RevisionControl",
    "Translation", "Calendar", "ContactManagement", "Database", "Dictionary", "Chart",
    "Email", "Finance", "FlowChart", "PDA", "ProjectManagement", "Presentation",
    "Spreadsheet", "WordProcessor", "2DGraphics", "VectorGraphics", "RasterGraphics",
    "3DGraphics", "Scanning", "OCR", "Photography", "Publishing", "Viewer", "TextTools",
    "DesktopSettings", "HardwareSettings", "Printing", "PackageManager", "Dialup",
    "InstantMessaging", "Chat", "IRCClient", "Feed", "FileTransfer", "HamRadio", "News",
    "P2P", "RemoteAccess", "Telephony", "TelephonyTools", "VideoConference", "WebBrowser",
    "WebDevelopment", "Midi", "Mixer", "Sequencer", "Tuner", "TV", "AudioVideoEditing",
    "Player", "Recorder", "DiscBurning", "ActionGame", "AdventureGame", "ArcadeGame",
    "BoardGame", "BlocksGame", "CardGame", "KidsGame", "LogicGame", "RolePlaying",
    "Shooter", "Simulation", "SportsGame", "StrategyGame", "Music", "Amusement",
    "Archiving", "Compression", "Electronics", "Emulator", "Engineering", "FileTools",
    "FileManager", "TerminalEmulator", "Filesystem", "Monitor", "Security",
    "Accessibility", "Calculator", "Clock", "TextEditor", "Documentation", "Adult",
    "Core", "KDE", "GNOME", "XFCE", "GTK", "Qt", "Motif", "Java", "ConsoleOnly",
    "Screensaver", "TrayIcon", "Applet", "Shell",
];

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn none() -> Selection {
        Selection { macos: false, windows: false, linux: false, snap: false, flatpak: false, appimage: false }
    }

    /// A Merged with a valid SVG and default effects; identity all empty.
    fn stub(sel: Selection) -> Merged {
        Merged {
            svg: Some("master.svg".into()),
            svg_exists: true,
            symbolic: None,
            id: None,
            name: None,
            exec: None,
            generic_name: None,
            comment: None,
            description: None,
            categories: Vec::new(),
            keywords: Vec::new(),
            startup_wm_class: None,
            mime_types: Vec::new(),
            license: None,
            metadata_license: None,
            homepage_url: None,
            developer_name: None,
            output_dir: "dist".into(),
            hicolor_sizes: vec![16, 32],
            squircle_power: 5.0,
            squircle_depth: true,
            gloss_strength: 0.35,
            depth_blur: 0.12,
            deb: false,
            sel,
        }
    }

    fn has_hard(p: &Problems, needle: &str) -> bool {
        p.hard.iter().any(|h| h.headline.contains(needle))
    }
    fn has_soft(p: &Problems, needle: &str) -> bool {
        p.soft.iter().any(|s| s.headline.contains(needle))
    }

    #[test]
    fn macos_only_requires_id_and_name_but_not_exec() {
        let sel = Selection { macos: true, ..none() };
        let p = evaluate(&stub(sel));
        assert!(has_hard(&p, "application id"));
        assert!(has_hard(&p, "application name"));
        assert!(!has_hard(&p, "exec command"));
        // No Linux metadata demanded when Linux isn't selected.
        assert!(!has_soft(&p, "description missing"));
    }

    #[test]
    fn windows_only_does_not_require_id() {
        let sel = Selection { windows: true, ..none() };
        let mut m = stub(sel);
        m.name = Some("App".into());
        let p = evaluate(&m);
        assert!(!has_hard(&p, "application id"));
        assert!(p.hard.is_empty());
    }

    #[test]
    fn linux_with_identity_warns_on_missing_metadata() {
        let sel = Selection { linux: true, ..none() };
        let mut m = stub(sel);
        m.id = Some("com.example.App".into());
        m.name = Some("App".into());
        m.exec = Some("app".into());
        let p = evaluate(&m);
        assert!(p.hard.is_empty(), "identity satisfied → no hard errors");
        assert!(has_soft(&p, "description missing"));
        assert!(has_soft(&p, "metadata_license missing"));
    }

    #[test]
    fn packaging_without_linux_errors() {
        let sel = Selection { macos: true, snap: true, ..none() };
        let mut m = stub(sel);
        m.id = Some("com.example.App".into());
        m.name = Some("App".into());
        let p = evaluate(&m);
        assert!(has_hard(&p, "--snap requires --linux"));
    }

    #[test]
    fn no_platform_selected_errors() {
        let p = evaluate(&stub(none()));
        assert!(has_hard(&p, "no platform selected"));
        // Field requirements stay quiet when nothing is selected.
        assert!(!has_hard(&p, "application id"));
    }

    #[test]
    fn non_reverse_dns_id_warns() {
        let sel = Selection { linux: true, ..none() };
        let mut m = stub(sel);
        m.id = Some("MyApp".into());
        m.name = Some("App".into());
        m.exec = Some("app".into());
        let p = evaluate(&m);
        assert!(has_soft(&p, "is not reverse-DNS"));
    }

    #[test]
    fn unregistered_category_warns() {
        let sel = Selection { linux: true, ..none() };
        let mut m = stub(sel);
        m.id = Some("com.example.App".into());
        m.name = Some("App".into());
        m.exec = Some("app".into());
        m.categories = vec!["Utility".into(), "Bogus".into()];
        let p = evaluate(&m);
        assert!(has_soft(&p, "unregistered categories: Bogus"));
    }

    #[test]
    fn bad_effects_error_only_for_macos() {
        let mut m = stub(Selection { macos: true, ..none() });
        m.id = Some("com.example.App".into());
        m.name = Some("App".into());
        m.gloss_strength = 5.0;
        assert!(has_hard(&evaluate(&m), "invalid gloss_strength"));

        // Same bad value, but macOS not selected → ignored.
        let mut w = stub(Selection { windows: true, ..none() });
        w.name = Some("App".into());
        w.gloss_strength = 5.0;
        assert!(!has_hard(&evaluate(&w), "invalid gloss_strength"));
    }
}