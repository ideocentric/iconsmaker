# iconsmaker

One SVG in. Every platform icon bundle out.

`iconsmaker` reads a single source SVG and a small TOML config file and generates
deployment-ready icon assets for macOS, Windows, and Linux — including packaging
variants for Snap, Flatpak, and AppImage — in a single command.

---

## Why iconsmaker

Most icon generators target macOS, Windows, and web favicons — and they almost
always start from a raster (PNG) image. You *can* press them into service for
Linux, but that's where it falls apart: you're left manually producing a spread
of exact sizes, applying strict reverse-DNS file names, reorganizing everything
into the freedesktop `hicolor` directory tree, and then hand-writing the
`.desktop` entry, the AppStream `metainfo.xml`, and a separate icon variant for
each packaging format (Snap, Flatpak, AppImage). The requirements are finite but
scattered across specs, and looking them all up and assembling them by hand for
every project is a slow, error-prone chore.

`iconsmaker` automates the whole thing from a single **1024 × 1024 SVG**. One
command renders every required size, applies the platform shaping (like the macOS
squircle) automatically, names and files each asset exactly where each platform
expects it, and generates the accompanying metadata — producing every permutation
for macOS, Windows, and Linux (including Snap/Flatpak/AppImage) with no manual
resizing, renaming, or reorganizing. And because the source is vector rather than
a raster image, small sizes stay crisp instead of turning to mush.

---

## What it generates

| Platform | Output | Notes |
|---|---|---|
| macOS | `dist/macos/AppName.icns` | 7 ICNS elements (16 – 1024 px), Apple squircle mask applied |
| Windows | `dist/windows/AppName.ico` | 7 sizes (16 – 256 px), PNG + BMP per ICO spec |
| Linux | `dist/linux/hicolor/` | 9 PNG sizes + scalable SVG + optional symbolic SVG |
| Linux | `dist/linux/<id>.desktop` | freedesktop Desktop Entry |
| Linux | `dist/linux/<id>.metainfo.xml` | AppStream MetaInfo for GNOME Software / Discover |
| Snap | `dist/snap/snap/gui/` | 256 × 256 store icon + desktop file |
| Flatpak | `dist/flatpak/app/share/` | Full hicolor tree + metadata under the Flatpak prefix |
| AppImage | `dist/appimage/<Name>.AppDir/` | Top-level icon, `.DirIcon`, inner hicolor tree |

---

## Installation

Prebuilt native installers are published to [GitHub Releases](../../releases) for
each tagged version.

### macOS

```bash
brew tap ideocentric/tap
brew install iconsmaker
```

Or download `iconsmaker-<version>-universal-apple-darwin.tar.gz` from the release
and place `iconsmaker` on your `PATH` — it's a universal binary that runs natively
on both Apple Silicon and Intel Macs.

### Linux

`x86_64` (`amd64`) and `arm64` (`aarch64`) packages are published — pick the one
matching `uname -m`.

```bash
# Debian / Ubuntu
sudo dpkg -i iconsmaker_<version>_amd64.deb      # or _arm64.deb

# Fedora / RHEL / openSUSE
sudo rpm -i iconsmaker-<version>.x86_64.rpm      # or .aarch64.rpm
```

Both packages install the binary to `/usr/bin/iconsmaker` and the man page to
`/usr/share/man/man1`.

### Windows

```powershell
winget install ideocentric.iconsmaker
```

winget picks the right build for your CPU automatically. To install by hand,
download and run `iconsmaker-<version>-x86_64.msi` or `iconsmaker-<version>-arm64.msi`
(installs to *Program Files* and adds itself to `PATH`).

### From source

```bash
cargo install --path .        # or: cargo build --release
```

Requires a Rust 1.85+ toolchain. For full prerequisites, per-platform notes, and
how to build a universal macOS binary or the installers, see
**[BUILDING.md](BUILDING.md)**.

### Documentation / man page

On macOS and Linux the package installs a man page:

```bash
man iconsmaker
```

On Windows the MSI installs `iconsmaker.html` alongside the binary. On any
platform you can also print the man page directly:

```bash
iconsmaker --print-man-page | sudo tee /usr/share/man/man1/iconsmaker.1
```

See [`packaging/`](packaging/) for how the installers are built.

---

## Prerequisites

Choose one:

**Docker** (recommended — no Rust toolchain needed)
```
Docker 20.10+
```

**Cargo** (native build)
```
Rust 1.85+ (edition 2024)
```

Full build instructions and per-platform prerequisites are in
[BUILDING.md](BUILDING.md).

---

## Quick start

### Via Docker

```bash
# 1. Clone and build the image once
git clone https://github.com/yourname/iconsmaker
cd iconsmaker
docker build -t iconsmaker:latest .

# 2. Run against any project
docker run --rm \
  -v "/path/to/myproject:/workspace" \
  iconsmaker:latest \
  --config icons/icons.toml
```

### Via Cargo

```bash
# From your project directory
cargo run --manifest-path /path/to/iconsmaker/Cargo.toml -- \
  --config icons/icons.toml
```

---

## Two ways to run

`iconsmaker` can be driven by a config file **or** entirely by flags, and the two
compose — **flags override the config file**.

**Config mode** (IDE / toolchain): keep an `icons.toml` in your project and run it
as part of your build. If `--config` is omitted, `./icons.toml` is used when
present.

```bash
iconsmaker --config icons/icons.toml
```

**Flag mode** (web / CI backend): pass everything on the command line — ideal when
a UI collects the parameters and forwards them. Select platforms with
`--macos --windows --linux` (or `--all`):

```bash
iconsmaker -i logo.svg --all \
  --id com.example.MyApp --name "My App" --exec "myapp %F" \
  --comment "One-line summary" --description "Longer store paragraph." \
  --categories Utility --license MIT --metadata-license CC0-1.0 \
  --developer-name "Me"
```

Passing **any** platform/packaging flag switches selection to the CLI, so a lone
`--macos` builds *only* macOS (it does not also pick up platforms from the config).

Run with **no arguments** (and no `icons.toml` in the working directory) and
`iconsmaker` prints its help and exits `0`, like `docker` or `kubectl`. An
*incomplete* invocation — flags that don't add up to a buildable target — still
reports what's missing and exits non-zero.

### Required fields depend on the platforms you select

Requirements form a grid. iconsmaker reports every missing field at once, naming
the selected target that needs it, why, and how to supply it (both flag and
config form). **Hard** = blocks; **Soft** = warns (and becomes an error under
`--strict`).

| Field | macOS | Windows | Linux | Packaging\* |
|---|---|---|---|---|
| input SVG (`-i` / `[input] svg`) | Hard | Hard | Hard | Hard |
| `--name` | Hard | Hard | Hard | Hard |
| `--id` (reverse-DNS) | Hard | — | Hard | Hard |
| `--exec` | — | — | Hard | Hard |
| `--comment`, `--description`, `--categories`, `--metadata-license`, `--license`, `--developer-name` | — | — | Soft | Soft |

\* Packaging (`--snap` / `--flatpak` / `--appimage`) **requires `--linux`** and
inherits all Linux requirements.

Format lints (Soft): a non-reverse-DNS `--id` and unregistered `--categories`
values are flagged as warnings.

Every flag mirrors a config key (e.g. `--metadata-license` ⇄ `[app]
metadata_license`); run `iconsmaker --help` for the full list.

---

## Recommended project layout

Place your icon assets in an `icons/` subdirectory of your project:

```
myproject/
├── icons/
│   ├── master.svg          ← your 1024 × 1024 source artwork  (required)
│   ├── symbolic.svg        ← monochrome tray/panel variant     (optional)
│   └── icons.toml          ← configuration
├── dist/                   ← generated output (add to .gitignore)
└── ... (project source)
```

The `dist/` directory is entirely generated — never commit it.

---

## Preparing your artwork

### Base SVG (`master.svg`)

| Property | Requirement |
|---|---|
| Format | SVG 1.1 |
| Viewbox | Square — `1024 1024` strongly recommended |
| Text | **Must be converted to outlines.** The renderer has no font loader; text elements that reference font families will not render. Export "text as paths" / "outline text" from your vector editor. |
| Embedded images | Self-contained raster elements (base64 `<image>`) are supported. External file references (`xlink:href="file.png"`) are not. |
| Filters | The `resvg` engine supports a useful subset of SVG filters: `feGaussianBlur`, `feOffset`, `feFlood`, `feComposite`, `feMerge`, `feBlend`, `feColorMatrix`, and `feSpecularLighting`. Complex filter chains (e.g. `feConvolveMatrix`) may be ignored. |
| Size | Keep under 2 MB. Highly complex paths slow down rasterization. |

### What NOT to include in the master SVG

- **Drop shadows** — macOS and Linux add their own at the system level.
- **Rounded corners / squircle clipping** — the tool applies Apple's squircle mask automatically for macOS.
- **Platform-specific layering or gloss** — the `[effects]` section handles depth shading.

Design the artwork as a flat square. The tool handles all platform shaping.

### Symbolic SVG (`symbolic.svg`)

Used on Linux for system trays, notification areas, and high-contrast modes.

- Single colour — use `#363636` or `currentColor`
- Designed on a **16 × 16** grid (simplified silhouette of the full icon)
- No fills outside the silhouette; no gradients
- Place in `hicolor/symbolic/apps/` automatically when `symbolic_svg` is set

### Apple squircle shape

The macOS squircle mask is **generated analytically** at every icon size — there
is no shape file to supply or reference. Its roundness is controlled by the
`squircle_power` value in the `[effects]` section (default `5.0`, macOS-like).
See the `[effects]` reference below.

---

## `icons.toml` reference

Create this file in your `icons/` directory.
All fields in `[app]` are used for metadata generation (`.desktop`, `metainfo.xml`).

```toml
# ── Application identity ───────────────────────────────────────────────────────
[app]
# Reverse-DNS identifier — used as the filename stem for every asset and as the
# Flatpak / Snap / D-Bus application ID. Must be consistent across all files.
id            = "com.example.MyApp"

name          = "My App"
generic_name  = "My Application Type"   # e.g. "Audio Editor", "Image Viewer"
comment       = "One-line summary"       # .desktop Comment=, AppStream <summary>
exec          = "myapp %F"              # .desktop Exec= — shell command to launch

# freedesktop.org menu categories:
# https://specifications.freedesktop.org/menu-spec/latest/apa.html
categories    = ["Category", "SubCategory"]
keywords      = ["keyword1", "keyword2"]

# Must match your window's WM_CLASS so docks/taskbars group windows correctly.
startup_wm_class = "myapp"

mime_types    = ["application/x-myapp"]  # Leave empty if no file associations

license          = "GPL-3.0-or-later"   # SPDX identifier for your app
metadata_license = "CC0-1.0"            # License for the metadata itself

description   = "Longer paragraph for store listings."
homepage_url  = "https://example.com"
developer_name = "Your Name or Studio"

# ── Input assets ──────────────────────────────────────────────────────────────
[input]
svg          = "icons/master.svg"      # Path relative to the working directory

# The macOS squircle mask is generated analytically — no shape file needed.
# Tune its roundness via [effects] squircle_power below.

# Monochrome SVG for Linux trays (optional):
# symbolic_svg = "icons/symbolic.svg"

# ── Output location ───────────────────────────────────────────────────────────
[output]
dir = "dist"

# ── Platform targets ──────────────────────────────────────────────────────────
[platforms]
macos   = true
windows = true
linux   = true

# ── Linux PNG sizes ───────────────────────────────────────────────────────────
[linux]
# Standard freedesktop hicolor sizes. 48 is the minimum required; keep all of
# these for full desktop-environment compatibility.
hicolor_sizes = [16, 22, 24, 32, 48, 64, 128, 256, 512]

# ── Visual effects (macOS) ────────────────────────────────────────────────────
[effects]
# SDF bevel: simulates overhead lighting on the squircle boundary.
# Rim highlight on top, subtle shadow on bottom. Applied before masking.
squircle_depth = true
gloss_strength = 0.35   # 0.0 = flat, 1.0 = maximum. 0.35 is a natural default.
depth_blur     = 0.12   # Gaussian blur on the lighting maps.
                         # 0.00 = no smoothing (may show staircase artifacts)
                         # 0.12 = default (smooth without losing edge definition)
                         # 0.30 = very soft

# ── Packaging formats ─────────────────────────────────────────────────────────
[packaging]
snap     = false   # Generates dist/snap/snap/gui/
flatpak  = false   # Generates dist/flatpak/app/share/
appimage = false   # Generates dist/appimage/<Name>.AppDir/
deb      = false   # (reserved — not yet implemented)
```

---

## Running

### Docker

```bash
# Run from your project root, mounting it as /workspace:
docker run --rm \
  -v "$(pwd):/workspace" \
  iconsmaker:latest \
  --config icons/icons.toml

# With verbose output:
docker run --rm \
  -v "$(pwd):/workspace" \
  iconsmaker:latest \
  --config icons/icons.toml --verbose

# Using the included docker-compose.yml (from the iconsmaker directory):
PROJECT_DIR=/path/to/myproject \
  docker compose run --rm iconsmaker --config icons/icons.toml
```

### Cargo

```bash
# From your project directory:
cargo run \
  --manifest-path /path/to/iconsmaker/Cargo.toml \
  -- --config icons/icons.toml

# Or if iconsmaker is installed on PATH:
iconsmaker --config icons/icons.toml
```

---

## Output reference

After a successful run with all platforms enabled:

```
dist/
├── macos/
│   └── MyApp.icns                     macOS icon bundle
├── windows/
│   └── MyApp.ico                      Windows icon (multi-size)
├── linux/
│   ├── com.example.MyApp.desktop      freedesktop Desktop Entry
│   ├── com.example.MyApp.metainfo.xml AppStream MetaInfo
│   └── hicolor/
│       ├── 16x16/apps/com.example.MyApp.png
│       ├── 22x22/apps/ ...
│       ├── 24x24/apps/ ...
│       ├── 32x32/apps/ ...
│       ├── 48x48/apps/ ...
│       ├── 64x64/apps/ ...
│       ├── 128x128/apps/ ...
│       ├── 256x256/apps/ ...
│       ├── 512x512/apps/ ...
│       └── scalable/apps/com.example.MyApp.svg
├── snap/
│   └── snap/gui/
│       ├── icon.png                   256 × 256 Snap Store icon
│       └── com.example.MyApp.desktop
├── flatpak/
│   └── app/share/
│       ├── applications/com.example.MyApp.desktop
│       ├── metainfo/com.example.MyApp.metainfo.xml
│       └── icons/hicolor/ ...         (full hicolor tree)
└── appimage/
    └── MyApp.AppDir/
        ├── com.example.MyApp.png      top-level icon
        ├── .DirIcon -> ...            symlink (Linux only)
        ├── com.example.MyApp.desktop
        └── usr/share/ ...
```

### Installing Linux assets (tarball / manual)

```bash
# Icons
rsync -a dist/linux/hicolor/ /usr/share/icons/hicolor/
gtk-update-icon-cache -f -t /usr/share/icons/hicolor

# Desktop integration
install -Dm644 dist/linux/com.example.MyApp.desktop \
  /usr/share/applications/com.example.MyApp.desktop
install -Dm644 dist/linux/com.example.MyApp.metainfo.xml \
  /usr/share/metainfo/com.example.MyApp.metainfo.xml

# Validate
desktop-file-validate /usr/share/applications/com.example.MyApp.desktop
appstreamcli validate /usr/share/metainfo/com.example.MyApp.metainfo.xml
```

---

## Visual effects

The `[effects]` section applies a depth treatment to macOS icons only,
simulating overhead lighting on the squircle boundary.

```
gloss_strength  controls how strong the lighting contrast is
depth_blur      controls how smoothly the lighting fades across the edge band
```

The bevel band is fixed at 7 % of the icon's shorter dimension — about 72 px
at 1024 px, 18 px at 256 px. Edge pixels facing the light source (upper-left)
are brightened via Screen blend; pixels facing away are darkened via Multiply.

**Tuning guide:**

| `gloss_strength` | Effect |
|---|---|
| `0.15` | Barely perceptible — suitable for very dark or very light icons |
| `0.35` | Default — natural overhead light on a physical surface |
| `0.55` | Dramatic — similar to an app icon under strong directional lighting |

| `depth_blur` | Effect |
|---|---|
| `0.00` | Raw edge sampling — may show subtle pixel-level staircase |
| `0.12` | Default — artifacts smoothed, edge still defined |
| `0.25` | Soft — good for icons with gradients near the squircle boundary |

Set `squircle_depth = false` for a completely flat icon (no lighting).

---

## Build system integration

### CMake

Add icon generation as a custom target that runs before the app bundle step.

```cmake
# Find the tool (adjust if using Docker or a specific install path)
find_program(ICONSMAKER_BIN iconsmaker
    HINTS "$ENV{HOME}/.cargo/bin" "/usr/local/bin"
)

set(ICON_CONFIG  "${CMAKE_SOURCE_DIR}/icons/icons.toml")
set(ICON_SVG     "${CMAKE_SOURCE_DIR}/icons/master.svg")
set(ICON_ICNS    "${CMAKE_SOURCE_DIR}/dist/macos/${PROJECT_NAME}.icns")
set(ICON_ICO     "${CMAKE_SOURCE_DIR}/dist/windows/${PROJECT_NAME}.ico")

if(ICONSMAKER_BIN)
    add_custom_command(
        OUTPUT  "${ICON_ICNS}" "${ICON_ICO}"
        COMMAND "${ICONSMAKER_BIN}" --config "${ICON_CONFIG}"
        WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
        DEPENDS "${ICON_SVG}" "${ICON_CONFIG}"
        COMMENT "Generating platform icon bundles"
        VERBATIM
    )
    add_custom_target(icons DEPENDS "${ICON_ICNS}" "${ICON_ICO}")
else()
    message(STATUS "iconsmaker not found — skipping icon generation")
endif()
```

#### Via Docker (no local install needed)

```cmake
add_custom_command(
    OUTPUT  "${ICON_ICNS}" "${ICON_ICO}"
    COMMAND docker run --rm
            -v "${CMAKE_SOURCE_DIR}:/workspace"
            iconsmaker:latest
            --config icons/icons.toml
    WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
    DEPENDS "${ICON_SVG}" "${ICON_CONFIG}"
    COMMENT "Generating platform icon bundles via Docker"
    VERBATIM
)
add_custom_target(icons DEPENDS "${ICON_ICNS}" "${ICON_ICO}")
```

Then in your app bundle target:
```cmake
add_dependencies(MyApp icons)
```

### Makefile

```makefile
ICONSMAKER ?= iconsmaker

.PHONY: icons
icons:
	$(ICONSMAKER) --config icons/icons.toml

dist/macos/MyApp.icns: icons/master.svg icons/icons.toml
	$(ICONSMAKER) --config icons/icons.toml
```

---

## Adding iconsmaker to a new project

Follow these steps for each new application:

### 1 — Prepare artwork

- Design your icon in a vector editor (Figma, Affinity Designer, Inkscape, Sketch).
- Canvas size: **1024 × 1024 px** square.
- Export as SVG with **text converted to outlines** (no font dependencies).
- Do not add rounded corners, drop shadows, or gloss — the tool handles these.
- Save as `icons/master.svg` in your project.

### 2 — Create `icons/icons.toml`

Start from the template in the **`icons.toml` reference** section above.
At minimum, fill in:

```toml
[app]
id             = "com.yourname.YourApp"   # unique reverse-DNS ID
name           = "Your App"
comment        = "One-line description"
exec           = "yourapp"

[input]
svg       = "icons/master.svg"
```

### 3 — Run and inspect

```bash
docker run --rm -v "$(pwd):/workspace" iconsmaker:latest --config icons/icons.toml --verbose
```

Open `dist/macos/YourApp.icns` in Preview (macOS) and `dist/linux/hicolor/256x256/apps/` in
your file manager to visually check the output.

### 4 — Tune effects

If the depth bevel is too strong or too subtle, adjust in `icons.toml`:
```toml
[effects]
gloss_strength = 0.35   # reduce for subtle, increase for dramatic
depth_blur     = 0.12
```

Re-run to see the change immediately.

### 5 — Validate Linux metadata

```bash
desktop-file-validate dist/linux/com.yourname.YourApp.desktop
appstreamcli validate dist/linux/com.yourname.YourApp.metainfo.xml
```

Fix any validation errors before shipping.

### 6 — Wire into your build system

See **Build system integration** above. Add `dist/` to your project's `.gitignore`.

### 7 — Per-packaging-format setup

Enable packaging formats in `icons.toml` as needed and follow the platform's
own guide for referencing the generated assets:

| Format | Generated assets location | Reference |
|---|---|---|
| Snap | `dist/snap/snap/gui/` | Copy to your snap source tree |
| Flatpak | `dist/flatpak/app/share/` | Copy under your Flatpak module prefix |
| AppImage | `dist/appimage/YourApp.AppDir/` | Merge into your AppDir before `appimagetool` |
| `.deb` | `dist/linux/hicolor/` + `.desktop` | List in `debian/<pkg>.install` |

---

## SVG design reference

### Colour palette guidance

| Element | Use |
|---|---|
| Background fill | Avoid pure black (#000) — use near-black like #1a1a1a for depth |
| Primary colour | Should read clearly at 16 × 16 px — test at small sizes |
| High-contrast content | Avoid very similar colours next to each other; they merge at small sizes |

### Testing at small sizes

The most common mistake in icon design is content that looks fine at 512 px
but is unreadable at 16 px. Check your 16 × 16 and 22 × 22 PNG outputs.
Simplify the SVG or increase contrast if small sizes look muddy.

### resvg filter support

The following SVG filter primitives are supported:
`feBlend`, `feColorMatrix`, `feComposite`, `feFlood`, `feGaussianBlur`,
`feImage`, `feMerge`, `feOffset`, `feSpecularLighting`, `feTurbulence` (basic).

Not supported: `feConvolveMatrix`, `feDiffuseLighting` (partial), `feMorphology`,
`feDisplacementMap`. Unsupported primitives are silently ignored.

### Further reading

- [Linux Application Icon Asset Guide](docs/linux-app-icon-asset-guide.md) —
  freedesktop hicolor layout, per-desktop styling (GNOME/KDE/Xfce/…), and the
  Snap / Flatpak / AppImage / `.deb` icon requirements.

---

## Bundled assets

`iconsmaker` ships as a single self-contained binary with no external asset
files. The macOS squircle mask is generated analytically at runtime from the
`[effects] squircle_power` value — there is no bundled shape SVG to install or
reference.

A sample source icon is included at `icons/master.svg` so you can try a build
immediately after cloning:

```bash
cargo run --release -- --config icons.toml
# assets appear under dist/
```

---

## License

`iconsmaker` is free software licensed under the **GNU General Public License
v3.0 or later** ([GPL-3.0-or-later](LICENSE)).

## Acknowledgments

Claude (Anthropic) was used extensively in the design and implementation of this
package — architecture, the Rust code, the packaging/installer pipeline, and this
documentation.