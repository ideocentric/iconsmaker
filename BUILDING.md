# Building iconsmaker from source

`iconsmaker` is a pure-Rust project. Building it produces a **single
self-contained binary** with no runtime dependencies (on Linux it links only
glibc). None of the dependencies require system C libraries — no `fontconfig`,
`freetype`, `openssl`, `cmake`, or `pkg-config`. The only non-Rust thing you need
is a **C linker**, which every platform's standard build tools provide.

- [1. Prerequisites](#1-prerequisites)
- [2. Get the source](#2-get-the-source)
- [3. Build](#3-build)
- [4. Verify the build](#4-verify-the-build)
- [5. Install](#5-install)
- [Platform callouts](#platform-callouts)
- [Building the man page & installers](#building-the-man-page--installers)
- [Building with Docker (no toolchain)](#building-with-docker-no-toolchain)
- [Troubleshooting](#troubleshooting)

---

## 1. Prerequisites

| Requirement | Version | Notes |
|---|---|---|
| Rust toolchain | **1.85+** | The crate uses **edition 2024**, which requires Rust 1.85 or newer. |
| A C linker | any | Provided by your platform's standard build tools (below). Used only to link the final binary. |

Install Rust via [rustup](https://rustup.rs):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup update            # ensure >= 1.85 for edition 2024
```

Verify:

```bash
rustc --version          # must be 1.85.0 or newer
cargo --version
```

The C linker comes from:

| Platform | Provides the linker | Install |
|---|---|---|
| macOS | Xcode Command Line Tools | `xcode-select --install` |
| Debian / Ubuntu | `build-essential` (gcc) | `sudo apt install build-essential` |
| Fedora / RHEL | gcc | `sudo dnf install gcc` (or `sudo dnf group install "Development Tools"`) |
| Arch | `base-devel` | `sudo pacman -S base-devel` |
| openSUSE | gcc | `sudo zypper install gcc` |
| Windows | MSVC Build Tools | See [Windows callout](#windows) |

---

## 2. Get the source

```bash
git clone https://github.com/ideocentric/iconsmaker.git
cd iconsmaker
```

> The lockfile (`Cargo.lock`) is committed, so everyone builds the exact same
> dependency versions.

---

## 3. Build

**Release build** (optimized — what you want for real use):

```bash
cargo build --release
```

The binary is written to:

| Platform | Path |
|---|---|
| macOS / Linux | `target/release/iconsmaker` |
| Windows | `target\release\iconsmaker.exe` |

**Debug build** (faster to compile, slower to run — for development):

```bash
cargo build
# -> target/debug/iconsmaker
```

> The first build compiles ~110 dependency crates and takes a few minutes.
> Subsequent builds are incremental and fast.

---

## 4. Verify the build

Run the test suite:

```bash
cargo test
```

Then do a real run against the bundled sample icon (`icons/master.svg`) — this
exercises the full pipeline for every platform target. There are two equivalent
ways to invoke it:

**Option A — with the example config file** (`icons.toml`, included in the repo):

```bash
cargo run --release -- --config icons.toml
```

**Option B — with full command-line arguments** (no config file needed):

```bash
./target/release/iconsmaker -i icons/master.svg --all \
  --id com.example.Demo --name "Demo App" --exec demo
```

Either way you should see output like:

```text
iconsmaker — Example App
Linux  : 9 sizes → dist/linux/hicolor
         .desktop → dist/linux/com.example.ExampleApp.desktop
         metainfo → dist/linux/com.example.ExampleApp.metainfo.xml
Windows: 7 sizes → dist/windows/Example_App.ico
macOS  : 7 sizes, depth, masking → dist/macos/Example_App.icns
Done.
```

> **You do not need to create `dist/` or any of its subfolders first.**
> On a fresh clone `dist/` does not exist yet — iconsmaker creates the entire
> output tree itself on every run, so the command never fails because a target
> folder is missing.

Afterwards you'll find a populated `dist/`:

```text
dist/
├── macos/Example_App.icns
├── windows/Example_App.ico
└── linux/
    ├── com.example.ExampleApp.desktop
    ├── com.example.ExampleApp.metainfo.xml
    └── hicolor/
        ├── 16x16/apps/…   22x22/…   24x24/…   32x32/…   48x48/…
        ├── 64x64/…   128x128/…   256x256/…   512x512/…
        └── scalable/apps/com.example.ExampleApp.svg
```

Open a couple of the PNGs (e.g. `dist/linux/hicolor/256x256/apps/…`) to confirm
they look right. That's a successful build.

---

## 5. Install

Install the binary onto your `PATH` with Cargo:

```bash
cargo install --path .        # -> ~/.cargo/bin/iconsmaker
```

Or copy the release binary wherever you like:

```bash
sudo install -m755 target/release/iconsmaker /usr/local/bin/iconsmaker   # macOS/Linux
```

To install the man page (macOS/Linux):

```bash
iconsmaker --print-man-page | sudo tee /usr/share/man/man1/iconsmaker.1 >/dev/null
```

For prebuilt native installers (`.deb`/`.rpm`, Homebrew, `.msi`) instead of a
source build, see the [README installation section](README.md#installation).

---

## Platform callouts

### macOS

- Install the linker with `xcode-select --install` (a full Xcode install is not
  required).
- A plain `cargo build --release` targets your Mac's own architecture
  (`aarch64-apple-darwin` on Apple Silicon, `x86_64-apple-darwin` on Intel).
- **Universal binary** (runs natively on both Apple Silicon *and* Intel):

  ```bash
  rustup target add aarch64-apple-darwin x86_64-apple-darwin
  cargo build --release --target aarch64-apple-darwin
  cargo build --release --target x86_64-apple-darwin
  lipo -create -output iconsmaker \
    target/aarch64-apple-darwin/release/iconsmaker \
    target/x86_64-apple-darwin/release/iconsmaker
  lipo -archs iconsmaker        # -> x86_64 arm64
  ```

  (`lipo` ships with the Command Line Tools.)

### Linux

- Install a C linker via your distro's dev tools (see the table in
  [Prerequisites](#1-prerequisites)).
- The result links only glibc — there are **no** extra runtime package
  dependencies.
- **arm64 (aarch64):** a native `cargo build --release` on an arm64 host just
  works; no cross-compilation setup needed.
- To build `.deb` / `.rpm` packages, see
  [Building the man page & installers](#building-the-man-page--installers).

### Windows

- Rust's default toolchain is **MSVC**, which needs the Microsoft C++ build
  tools. Install **"Desktop development with C++"** from the
  [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022),
  then `cargo build --release`.
- Build and run in PowerShell or `cmd`; the binary is `target\release\iconsmaker.exe`.
- **GNU alternative** (no Visual Studio): use the MinGW-w64 toolchain:

  ```powershell
  rustup default stable-x86_64-pc-windows-gnu
  ```

- **arm64 Windows:** build natively on a `windows-11-arm` machine, or add the
  target and cross-compile:

  ```powershell
  rustup target add aarch64-pc-windows-msvc
  cargo build --release --target aarch64-pc-windows-msvc
  ```

---

## Building the man page & installers

These are optional and only needed if you're producing distributable artifacts.
The heavy lifting is automated in
[`.github/workflows/release.yml`](.github/workflows/release.yml); to do it by
hand, see [`packaging/README.md`](packaging/README.md). Quick reference:

| Artifact | Extra tool | Command |
|---|---|---|
| man page (`iconsmaker.1`) | — (built binary) | `iconsmaker --print-man-page > iconsmaker.1` |
| HTML help | `pandoc` | `pandoc -s -f man -t html iconsmaker.1 -o iconsmaker.html` |
| Debian `.deb` | `cargo-deb` | `cargo install cargo-deb && cargo deb` |
| RPM `.rpm` | `cargo-generate-rpm` | `cargo install cargo-generate-rpm && cargo generate-rpm` |
| Windows `.msi` | `cargo-wix` + [WiX](https://wixtoolset.org/) | `cargo install cargo-wix && cargo wix` |
| License audit | `cargo-deny` | `cargo install cargo-deny && cargo deny check licenses` |

> The `.deb`/`.rpm` builders expect the man page at `target/assets/iconsmaker.1` —
> generate it there first (see `packaging/README.md`).

---

## Building with Docker (no toolchain)

If you don't want a local Rust toolchain, build the binary in a container using
the bundled [`Dockerfile`](Dockerfile):

```bash
docker build -t iconsmaker:latest .
docker run --rm -v "$(pwd):/workspace" iconsmaker:latest --config icons.toml
```

This compiles inside the image and needs nothing on the host but Docker.

---

## Development tooling

For working on the code:

```bash
rustup component add rustfmt clippy
cargo fmt --all             # format
cargo clippy --all-targets  # lint
cargo test                  # unit tests
```

---

## Troubleshooting

| Symptom | Cause / fix |
|---|---|
| `edition2024` / "feature `edition2024` is required" | Rust older than 1.85. Run `rustup update`. |
| `error: linker \`cc\` not found` (Linux) | No C linker. Install `build-essential` / `gcc` (see Prerequisites). |
| `xcrun: error: invalid active developer path` (macOS) | Command Line Tools missing. Run `xcode-select --install`. |
| `link.exe not found` (Windows) | MSVC C++ build tools missing — install them, or switch to the GNU toolchain. |
| First build is slow | Normal: ~110 crates compile once, then builds are incremental. |
| `iconsmaker: source SVG not found` | Run from the repo root (so `icons/master.svg` / `icons.toml` resolve), or pass `-i /path/to/your.svg`. |