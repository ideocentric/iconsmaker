# Linux Application Icon Asset Guide

Coverage: Ubuntu (GNOME) as primary, plus KDE Plasma, Xfce, Cinnamon, MATE, Budgie, LXQt, Elementary OS. Includes packaging-format requirements for `.deb`, Snap, Flatpak, and AppImage.

---

## 1. The foundation: freedesktop.org `hicolor` theme

All major Linux desktops resolve application icons through the [freedesktop.org Icon Theme Specification](https://specifications.freedesktop.org/icon-theme/latest/). Every theme inherits from `hicolor` as the final fallback. If you ship a properly structured hicolor asset set, every desktop will pick it up — GNOME, KDE, Xfce, Cinnamon, MATE, Budgie, LXQt, Elementary, etc.

**Why this matters**: Don't waste time exporting per-desktop icon variants. Ship one hicolor set with multiple sizes plus a scalable SVG, and the desktops handle the rest via their own theme resolution.

### Install roots

| Scope | Path |
|---|---|
| System-wide | `/usr/share/icons/hicolor/` |
| Per-user | `~/.local/share/icons/hicolor/` |
| Packaging staging | `$DESTDIR/$prefix/share/icons/hicolor/` |
| Legacy fallback (avoid) | `/usr/share/pixmaps/` |

### Naming convention

Use **reverse-DNS notation** for the icon basename, matching your app ID (same string used in `.desktop` files, AppStream metainfo, Flatpak/Snap manifests, D-Bus names):

```
com.example.ExampleApp        ← good
ExampleApp                       ← acceptable but legacy
exampleapp                       ← acceptable, lower-case
```

Reverse-DNS avoids collisions in Flathub, the Snap Store, and across packaging formats. The same string becomes the filename stem of every asset.

### Directory layout

```
$prefix/share/icons/hicolor/
├── 16x16/apps/com.example.ExampleApp.png
├── 22x22/apps/com.example.ExampleApp.png      # KDE prefers this size
├── 24x24/apps/com.example.ExampleApp.png      # GNOME menus/panels
├── 32x32/apps/com.example.ExampleApp.png
├── 48x48/apps/com.example.ExampleApp.png      # ← MINIMUM REQUIRED
├── 64x64/apps/com.example.ExampleApp.png
├── 128x128/apps/com.example.ExampleApp.png    # GNOME canonical
├── 256x256/apps/com.example.ExampleApp.png
├── 512x512/apps/com.example.ExampleApp.png    # HiDPI
├── scalable/apps/com.example.ExampleApp.svg   # strongly recommended
└── symbolic/apps/com.example.ExampleApp-symbolic.svg
```

The 48×48 PNG is the only strict minimum per the spec. Everything else is "highly recommended" — and shipping the scalable SVG plus 128/256/512 PNGs is what you actually want in 2026, because HiDPI displays are now common.

### Symbolic icons

A monochrome SVG that the system recolors based on context (light/dark theme, high-contrast mode, panel state). GNOME uses these heavily for system trays, notifications, and at small sizes. KDE's Breeze theme also makes use of them. Required if you want first-class GNOME integration.

- Filename: `<app-id>-symbolic.svg`
- Single color (typically `#363636` or `currentColor`), no fills outside the silhouette
- Drawn on a 16×16 grid, simplified compared to the full icon
- Place in `hicolor/symbolic/apps/`

GNOME's [App Icon Preview](https://flathub.org/apps/org.gnome.design.AppIconPreview) tool generates both the full and symbolic variants from a single SVG template.

---

## 2. Per-desktop style notes

### GNOME (Ubuntu 22.04+, Fedora Workstation, Endless OS, etc.)

Authoritative reference: [GNOME HIG — App Icons](https://developer.gnome.org/hig/guidelines/app-icons.html).

- Canvas: 128×128, but artwork shouldn't fill the canvas — leave breathing room per the template guides.
- Style: simple, geometric, **slightly skeuomorphic**. Not flat. Depth comes from combining a "top" surface with a darker "front" profile (typically ≤4px tall) sitting on the baseline.
- Materials: skeuomorphic textures (wood, glass, metal) allowed; gradients reserved for curved surfaces; flat colors on straight surfaces.
- Shadows: avoid outside the silhouette — the system adds those programmatically based on context. Internal shadows OK to differentiate layers, light source pointing straight down.
- Avoid extreme aspect ratios. Avoid text in the icon.
- Use the [App Icon Preview](https://flathub.org/apps/org.gnome.design.AppIconPreview) tool — it generates the template, previews in real contexts, and exports both full and symbolic SVGs.

### KDE Plasma (Kubuntu, openSUSE, KDE Neon, Fedora KDE Spin)

Authoritative reference: [KDE HIG — Icon Design](https://develop.kde.org/hig/style/icons/) and the [Breeze icon repository](https://invent.kde.org/frameworks/breeze-icons).

- Style: **flat, geometric, monoline-influenced**. The opposite design language from GNOME's slightly-skeuomorphic style.
- Standard sizes: 16, 22, 32, 48, 64, 128, 256 (KDE uses 22×22 in many places where GNOME uses 24×24).
- Two-color or limited-palette designs are common; outlined shapes more than filled ones.
- Symbolic icons (`-symbolic` suffix) work but Breeze's monochrome treatment uses a `-symbolic` *and* the `colors` group inside the SVG (via the [colorscheme stylesheet mechanism](https://develop.kde.org/hig/style/icons/colorful/)).
- KDE will still display your hicolor icon fine — but if you want true Breeze-style integration, ship a second SVG drawn in that style.

### Xfce (Xubuntu, Manjaro Xfce, MX Linux)

- No dedicated style guide — Xfce inherits from whatever icon theme is active (commonly Adwaita, Papirus, Elementary, or its own Greybird).
- Ship the hicolor set. That's it. It works.

### Cinnamon (Linux Mint, LMDE)

- Cinnamon uses its own Mint-X / Mint-Y themes by default.
- Hicolor fallback works fine; no custom asset needed.

### MATE (Ubuntu MATE)

- Inherits from GNOME 2 conventions. Hicolor works.

### Budgie (Ubuntu Budgie, Solus)

- Defaults to Papirus icon theme. Hicolor fallback works.

### LXQt (Lubuntu)

- Defaults to Breeze or Oxygen. Hicolor fallback works.

### Elementary OS

Authoritative reference: [elementary HIG — Iconography](https://docs.elementary.io/hig/reference/iconography).

- Distinctive house style: colorful, friendly, **isometric** for some app icons, with a strong front-facing design language for others.
- Their tooling and guidelines are well-documented and worth following if you're publishing to AppCenter.
- For broader distribution, the hicolor set is still the baseline; Elementary's icon theme will substitute its own icons for first-party apps but use yours for third-party.

**Practical takeaway**: Design once in GNOME's style (it's the most permissive and Ubuntu's default), ship hicolor, optionally add a Breeze variant if KDE is a major target and an Elementary variant if AppCenter publishing matters.

---

## 3. The `.desktop` file (essential glue)

Without this, your icon won't appear in menus or launchers, even if every PNG is in the right place. The `Icon=` field is what links the application entry to your hicolor assets.

**Install path**: `$prefix/share/applications/com.example.ExampleApp.desktop`

```ini
[Desktop Entry]
Type=Application
Version=1.0
Name=Example App
GenericName=Example Application
Comment=One-line description of the application
Exec=exampleapp %F
Icon=com.example.ExampleApp
Terminal=false
Categories=AudioVideo;Audio;Music;
Keywords=example;icons;
StartupNotify=true
StartupWMClass=exampleapp
MimeType=application/x-exampleapp;
```

Key points:

- `Icon=` takes the **basename only**, no path, no extension. The desktop's icon-resolution machinery finds the right size from the hicolor tree.
- `StartupWMClass` must match the window's WM_CLASS so the launcher correctly groups the app's windows under its icon in docks/taskbars. Critical for proper Alt-Tab and dock behavior.
- `Categories` should follow the [freedesktop menu specification](https://specifications.freedesktop.org/menu-spec/latest/apa.html). For an audio app: `AudioVideo;Audio;` (the first is the required main category).

Validate with: `desktop-file-validate com.example.ExampleApp.desktop`

---

## 4. AppStream MetaInfo (recommended)

Modern app stores (GNOME Software, KDE Discover, Flathub, Snap Store) read [AppStream MetaInfo](https://www.freedesktop.org/software/appstream/docs/) for store listings, including the icon reference.

**Install path**: `$prefix/share/metainfo/com.example.ExampleApp.metainfo.xml`

```xml
<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
  <id>com.example.ExampleApp</id>
  <name>Example App</name>
  <summary>One-line summary of the application</summary>
  <metadata_license>CC0-1.0</metadata_license>
  <project_license>GPL-3.0-or-later</project_license>
  <launchable type="desktop-id">com.example.ExampleApp.desktop</launchable>
  <icon type="stock">com.example.ExampleApp</icon>
  <!-- ... other fields ... -->
</component>
```

Validate with: `appstreamcli validate com.example.ExampleApp.metainfo.xml`

---

## 5. Packaging-format requirements

### `.deb` (Ubuntu/Debian native)

Standard hicolor layout under `/usr/share/icons/hicolor/`. Your `debian/install` or `debian/<package>.install` file lists the icon paths. Trigger an icon cache update via `dh_icons` (handled automatically by debhelper compat ≥10).

Required: nothing beyond the hicolor set + `.desktop` file.

### Snap

Snap has **two icon concepts**:

1. **Store icon** — defined by `icon:` in `snapcraft.yaml`. Per the [snapcraft.yaml reference](https://documentation.ubuntu.com/snapcraft/latest/reference/project-file/snapcraft-yaml/), icon size can be between 40×40 and 512×512 pixels, 256×256 is recommended, and the file should be less than 256 KB. Convention is `snap/gui/icon.png` or `snap/gui/icon.svg`.
2. **Desktop icon** — referenced from the bundled `.desktop` file (placed in `snap/gui/<app-name>.desktop` and installed to `$SNAP/meta/gui/`). This uses the standard hicolor lookup *within the snap's confined filesystem*.

```yaml
# snapcraft.yaml
name: exampleapp
icon: snap/gui/exampleapp.png       # 256x256 recommended, < 256 KB

apps:
  exampleapp:
    command: bin/exampleapp
    desktop: snap/gui/exampleapp.desktop
```

For the in-snap `.desktop` file, the `Icon=` line must reference a path inside the snap, typically `Icon=${SNAP}/meta/gui/icon.png`, which `snapcraft` rewrites at build time.

### Flatpak

[Flatpak conventions](https://docs.flatpak.org/en/latest/conventions.html) require strict reverse-DNS app IDs. Icons install to the standard hicolor layout *inside* the flatpak prefix (`/app/share/icons/hicolor/...`), and the desktop file + metainfo go to `/app/share/applications/` and `/app/share/metainfo/`.

Flathub-specific requirements: ship a **128×128 PNG minimum**, and a scalable SVG is strongly recommended. The icon name in your metainfo and desktop file must match the app ID exactly.

A minimal manifest snippet:

```yaml
# com.example.ExampleApp.yml
app-id: com.example.ExampleApp
runtime: org.freedesktop.Platform
# ...
modules:
  - name: exampleapp
    buildsystem: simple
    build-commands:
      - install -Dm644 exampleapp.svg /app/share/icons/hicolor/scalable/apps/com.example.ExampleApp.svg
      - install -Dm644 exampleapp-128.png /app/share/icons/hicolor/128x128/apps/com.example.ExampleApp.png
      - install -Dm644 exampleapp-256.png /app/share/icons/hicolor/256x256/apps/com.example.ExampleApp.png
      - install -Dm644 com.example.ExampleApp.desktop /app/share/applications/com.example.ExampleApp.desktop
      - install -Dm644 com.example.ExampleApp.metainfo.xml /app/share/metainfo/com.example.ExampleApp.metainfo.xml
```

### AppImage

AppImage has its own integration model: when extracted/run, it can self-register by copying an icon and `.desktop` file to the user's `~/.local/share/`.

Required at the AppImage root:

- `<AppName>.desktop` — top-level
- `<AppName>.png` or `.svg` — top-level, **must match the `Icon=` value in the .desktop file** (basename, no path, no extension in the desktop file but the file at root needs an extension)
- `.DirIcon` — symlink to the icon, used by file managers when displaying the AppImage itself

Inside the AppDir (before squashing into AppImage):

```
ExampleApp.AppDir/
├── AppRun
├── exampleapp.desktop                          # top-level
├── exampleapp.svg                              # top-level, matches Icon= field
├── .DirIcon -> exampleapp.svg
└── usr/
    ├── bin/exampleapp
    └── share/
        ├── applications/exampleapp.desktop     # duplicate, for AppStream
        ├── metainfo/exampleapp.metainfo.xml
        └── icons/hicolor/
            ├── 128x128/apps/exampleapp.png
            ├── 256x256/apps/exampleapp.png
            └── scalable/apps/exampleapp.svg
```

`appimagetool` requires the `Categories=` key in the `.desktop` file or it will fail validation.

---

## 6. HiDPI and scaling

Unlike macOS (which uses `@2x` suffixed assets) or Windows (which uses scale-factor suffixes like `.scale-200.png`), Linux desktops don't use filename suffixes for HiDPI. They either:

- Pick the next-largest available size and scale down, or
- Use the SVG from `scalable/apps/`

**Practical guidance**: always ship the SVG plus 256×256 and 512×512 PNGs. The SVG handles arbitrary scale factors; the high-resolution PNGs handle desktops or apps that can't render SVGs (rare, but exists in some toolkits).

GNOME 47+ and KDE Plasma 6 both default to SVG rendering when available.

---

## 7. Post-install icon cache refresh

When you install icons into `hicolor/`, the icon cache must be regenerated, or the new icons won't appear until reboot:

```bash
gtk-update-icon-cache -f -t /usr/share/icons/hicolor
```

For `.deb` packaging, `dh_icons` calls this automatically via maintainer scripts. For Snap and Flatpak, the sandboxing handles this internally. For manual installs (your install script, makefile, or `xdg-icon-resource`):

```bash
# Manual icon registration (preferred for tarball installs)
xdg-icon-resource install --size 48 --novendor exampleapp-48.png exampleapp
xdg-icon-resource install --size 128 --novendor exampleapp-128.png exampleapp
xdg-icon-resource install --size 256 --novendor exampleapp-256.png exampleapp
xdg-desktop-menu install --novendor com.example.ExampleApp.desktop
```

---

## 8. Export workflow

From a single 1024×1024 SVG master:

```bash
#!/usr/bin/env bash
# export-linux-icons.sh
set -euo pipefail

APP_ID="com.example.ExampleApp"
MASTER_SVG="design/exampleapp-master.svg"
SYMBOLIC_SVG="design/exampleapp-symbolic.svg"
OUT="dist/icons/hicolor"

mkdir -p "$OUT"/{scalable,symbolic}/apps

# Copy scalable SVGs
cp "$MASTER_SVG"    "$OUT/scalable/apps/${APP_ID}.svg"
cp "$SYMBOLIC_SVG"  "$OUT/symbolic/apps/${APP_ID}-symbolic.svg"

# Rasterize PNGs at standard sizes
for size in 16 22 24 32 48 64 128 256 512; do
    mkdir -p "$OUT/${size}x${size}/apps"
    inkscape --export-type=png \
             --export-width="$size" \
             --export-height="$size" \
             --export-filename="$OUT/${size}x${size}/apps/${APP_ID}.png" \
             "$MASTER_SVG"
done

# Optimize PNGs (optional but recommended)
find "$OUT" -name "*.png" -exec optipng -quiet -o5 {} \;

echo "Done. Install with:"
echo "  rsync -a $OUT/ /usr/share/icons/hicolor/"
echo "  sudo gtk-update-icon-cache -f /usr/share/icons/hicolor"
```

Alternative rasterizers: `rsvg-convert` (faster, no GUI dependency), `magick` (ImageMagick), `resvg` (Rust, sharpest at small sizes).

---

## 9. Deliverable checklist

For a single application, your final asset bundle should contain:

**Source masters**
- `exampleapp-master.svg` — 1024×1024 viewBox, full-color, all detail layers
- `exampleapp-symbolic.svg` — 16×16 viewBox, monochrome, simplified

**Rasterized hicolor PNGs** (basename matches app ID)
- 16, 22, 24, 32, 48, 64, 128, 256, 512

**Metadata files**
- `com.example.ExampleApp.desktop`
- `com.example.ExampleApp.metainfo.xml`

**Per-packaging-format**
- Snap: `snap/gui/icon.png` (256×256, <256KB)
- Flatpak: hicolor 128, 256, scalable + manifest references
- AppImage: top-level icon + `.DirIcon` symlink
- `.deb`: nothing extra beyond hicolor + debian/install rules

**Validation commands** before shipping:
```bash
desktop-file-validate com.example.ExampleApp.desktop
appstreamcli validate com.example.ExampleApp.metainfo.xml
# For Flatpak:
flatpak run --command=appstream-util org.flatpak.Builder validate-relax *.metainfo.xml
```

---

## 10. Authoritative references

- [freedesktop.org Icon Theme Specification](https://specifications.freedesktop.org/icon-theme/latest/)
- [freedesktop.org Icon Naming Specification](https://specifications.freedesktop.org/icon-naming-spec/latest/)
- [freedesktop.org Desktop Entry Specification](https://specifications.freedesktop.org/desktop-entry-spec/latest/)
- [freedesktop.org Menu Specification (Categories)](https://specifications.freedesktop.org/menu-spec/latest/)
- [GNOME HIG — App Icons](https://developer.gnome.org/hig/guidelines/app-icons.html)
- [KDE HIG — Icon Design](https://develop.kde.org/hig/style/icons/)
- [Elementary HIG — Iconography](https://docs.elementary.io/hig/reference/iconography)
- [Snapcraft snapcraft.yaml reference](https://documentation.ubuntu.com/snapcraft/latest/reference/project-file/snapcraft-yaml/)
- [Flatpak Requirements & Conventions](https://docs.flatpak.org/en/latest/conventions.html)
- [AppImage packaging guide](https://docs.appimage.org/packaging-guide/index.html)
- [AppStream MetaInfo reference](https://www.freedesktop.org/software/appstream/docs/)
