use anyhow::{Context, Result};
use clap::{CommandFactory, Parser};
use std::path::Path;

mod cli;
mod config;
mod output;
mod packaging;
mod render;
mod validate;

use cli::Cli;
use config::RawConfig;

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.print_man_page {
        let cmd = Cli::command();
        let man = clap_mangen::Man::new(cmd);
        let mut stdout = std::io::stdout().lock();
        man.render(&mut stdout).context("failed to render man page")?;
        return Ok(());
    }

    // ── Config detection ────────────────────────────────────────────────────
    // Explicit --config must exist. Otherwise use ./icons.toml when present, or
    // fall back to an empty config (flags-only mode).
    let (raw, config_loaded) = match &cli.config {
        Some(path) => (RawConfig::load(path)?, true),
        None if Path::new("icons.toml").exists() => (RawConfig::load("icons.toml")?, true),
        None => (RawConfig::default(), false),
    };

    // ── Merge + validate ────────────────────────────────────────────────────
    // Validation failures render their own aggregated message; print and exit
    // without anyhow's "Error:" prefix.
    let config = match validate::resolve(&cli, raw, config_loaded) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    println!("iconsmaker — {}", config.app.name);
    if cli.verbose {
        println!("  id     : {}", config.app.id);
        println!("  source : {}", config.input.svg);
        println!("  output : {}", config.output.dir);
    }

    let svg_path = Path::new(&config.input.svg);
    let symbolic_svg = config.input.symbolic_svg.as_deref().map(Path::new);
    let out_dir      = Path::new(&config.output.dir);
    let app_id       = &config.app.id;
    let app_filename = config.app.name.replace(' ', "_");
    let squircle_n   = config.effects.squircle_power;

    let linux_dir = out_dir.join("linux");
    std::fs::create_dir_all(out_dir)
        .with_context(|| format!("cannot create output directory: {}", out_dir.display()))?;

    // ── Linux: hicolor tree + metadata ─────────────────────────────────────
    if config.platforms.linux {
        let sizes = &config.linux.hicolor_sizes;
        print!("Linux  : {} sizes", sizes.len());
        let buffers = render::rasterize::rasterize_as_pairs(svg_path, sizes)?;
        let hicolor_dir = linux_dir.join("hicolor");
        output::png::write_hicolor_tree(&buffers, svg_path, symbolic_svg, app_id, &hicolor_dir)
            .with_context(|| format!("failed writing hicolor tree to {}", hicolor_dir.display()))?;
        println!(" → {}", hicolor_dir.display());

        let desktop_dest = linux_dir.join(format!("{}.desktop", app_id));
        output::metadata::write_desktop_file(&config, &desktop_dest)
            .with_context(|| format!("failed writing .desktop file: {}", desktop_dest.display()))?;
        println!("         .desktop → {}", desktop_dest.display());

        let metainfo_dest = linux_dir.join(format!("{}.metainfo.xml", app_id));
        output::metadata::write_metainfo(&config, &metainfo_dest)
            .with_context(|| format!("failed writing metainfo: {}", metainfo_dest.display()))?;
        println!("         metainfo → {}", metainfo_dest.display());
    }

    // ── Windows: .ico ───────────────────────────────────────────────────────
    if config.platforms.windows {
        let sizes = output::ico::WINDOWS_SIZES;
        print!("Windows: {} sizes", sizes.len());
        let buffers = render::rasterize::rasterize_as_pairs(svg_path, sizes)?;
        let ico_dest = out_dir.join("windows").join(format!("{}.ico", app_filename));
        output::ico::write_ico(&buffers, &ico_dest)
            .with_context(|| format!("failed writing .ico: {}", ico_dest.display()))?;
        println!(" → {}", ico_dest.display());
    }

    // ── macOS: .icns  (depth → squircle mask) ──────────────────────────────
    if config.platforms.macos {
        let sizes = output::icns::MACOS_SIZES;
        print!("macOS  : {} sizes", sizes.len());
        let mut buffers = render::rasterize::rasterize_as_pairs(svg_path, sizes)?;

        // Depth shading must run BEFORE masking: lighting is computed against
        // the unmasked buffer so the squircle edge clips it cleanly.
        if config.effects.squircle_depth {
            print!(", depth");
            for (_, buf) in &mut buffers {
                render::effects::apply_squircle_depth(
                    buf,
                    squircle_n,
                    config.effects.gloss_strength,
                    config.effects.depth_blur,
                );
            }
        }

        // Squircle mask always applied for macOS — generated analytically.
        print!(", masking");
        for (_, buf) in &mut buffers {
            render::compose::apply_squircle_mask(buf, squircle_n);
        }

        let icns_dest = out_dir.join("macos").join(format!("{}.icns", app_filename));
        output::icns::write_icns(&buffers, &icns_dest)
            .with_context(|| format!("failed writing .icns: {}", icns_dest.display()))?;
        println!(" → {}", icns_dest.display());
    }

    // ── Packaging bundles ───────────────────────────────────────────────────
    if config.packaging.snap {
        print!("Snap   : bundling");
        packaging::snap::write_snap_bundle(&config, &linux_dir, out_dir)
            .context("failed writing Snap bundle")?;
        println!(" → {}", out_dir.join("snap").display());
    }

    if config.packaging.flatpak {
        print!("Flatpak: bundling");
        packaging::flatpak::write_flatpak_bundle(&config, &linux_dir, out_dir)
            .context("failed writing Flatpak bundle")?;
        println!(" → {}", out_dir.join("flatpak").display());
    }

    if config.packaging.appimage {
        print!("AppImage: bundling");
        packaging::appimage::write_appimage_bundle(&config, &linux_dir, out_dir)
            .context("failed writing AppImage bundle")?;
        println!(
            " → {}",
            out_dir
                .join("appimage")
                .join(format!("{}.AppDir", app_filename))
                .display()
        );
    }

    println!("Done.");
    Ok(())
}