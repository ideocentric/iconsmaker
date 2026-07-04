# COPR project text

Copy-paste text for the COPR project's **Description** and **Instructions** fields
(both render Markdown). Reusable for other listings (PPA/AUR/etc.) too.

## Description

iconsmaker turns a single 1024×1024 SVG into complete, deployment-ready
application icon sets. For Linux it emits the freedesktop hicolor PNG tree
(16–512 px) plus a scalable SVG, a .desktop entry, and an AppStream metainfo.xml;
it also produces .icns (macOS) and .ico (Windows), with optional
Snap/Flatpak/AppImage scaffolding. One self-contained Rust CLI — run it once and
get every size, name, and layout each platform expects, no manual resizing or
reorganizing.

## Instructions

    sudo dnf copr enable ideocentric/iconsmaker
    sudo dnf install iconsmaker

Then `man iconsmaker` or `iconsmaker --help`. Homepage & full docs:
https://github.com/ideocentric/iconsmaker