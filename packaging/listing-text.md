# Listing text

Copy-paste text for package-registry / repo listings. One shared **Description**
(kept consistent across channels) plus per-channel install **Instructions**.

## Description (shared — COPR, PPA, etc.)

iconsmaker generates deployment-ready application icon sets from a single
1024×1024 SVG: the freedesktop hicolor PNG tree (16–512 px) plus a scalable SVG,
a .desktop entry, and an AppStream metainfo.xml — and it also produces .icns
(macOS) and .ico (Windows), with optional Snap/Flatpak/AppImage scaffolding. One
self-contained Rust CLI: run it once and get every size, name, and layout each
platform expects, with no manual resizing or reorganizing.

## Short description (winget / crates.io / one-liners)

Generate platform icon bundles (macOS, Windows, Linux) from a single SVG

## Instructions — Fedora COPR

    sudo dnf copr enable ideocentric/iconsmaker
    sudo dnf install iconsmaker

Then `man iconsmaker` or `iconsmaker --help`. Homepage & full docs:
https://github.com/ideocentric/iconsmaker

## Instructions — Ubuntu PPA

    sudo add-apt-repository ppa:ideocentric/iconsmaker
    sudo apt update
    sudo apt install iconsmaker

Homepage & source: https://github.com/ideocentric/iconsmaker