# Packaging

Native installers for iconsmaker, produced by
[`.github/workflows/release.yml`](../.github/workflows/release.yml) when a `v*`
tag is pushed. Everything here is generated from a single source: the binary,
plus the man page emitted by `iconsmaker --print-man-page`.

**Architectures:** Linux and Windows ship both `x86_64` and `arm64` (built on
native `ubuntu-24.04-arm` / `windows-11-arm` runners); macOS is a single
universal (arm64 + x86_64) binary.

| Platform | Artifact | Source in this dir |
|---|---|---|
| Linux (Debian/Ubuntu) | `.deb` (prebuilt) | `[package.metadata.deb]` in `Cargo.toml` (`cargo deb`) |
| Linux (Ubuntu PPA) | `.deb` (source build) | `debian/` (see `debian/README.md`) |
| Linux (Fedora/RHEL) | `.rpm` (prebuilt) | `[package.metadata.generate-rpm]` in `Cargo.toml` (`cargo generate-rpm`) |
| Linux (Fedora COPR) | `.rpm` (source build) | `copr/iconsmaker.spec` |
| macOS | Homebrew formula | `homebrew/iconsmaker.rb.tmpl` |
| Windows | `.msi` | `../wix/main.wxs` (`cargo wix`) |
| Windows | winget manifest | `winget/ideocentric.iconsmaker.*.yaml` |
| Windows | Scoop manifest | `ideocentric/scoop-bucket` repo |
| Arch | AUR `PKGBUILD` | `aur/` (see `aur/README.md`) |

## Documentation artifacts

- **man page** (`iconsmaker.1`) ships in the `.deb`/`.rpm` and the Homebrew
  formula → `man iconsmaker`.
- **HTML** (`iconsmaker.html`, generated from the man page with `pandoc`) ships
  in the Windows MSI and as a Release asset.

## One-time setup before the first release

The workflow itself is repo-agnostic (URLs come from `${{ github.repository }}`),
but three things must be set once the GitHub repo exists:

1. **`Cargo.toml` `repository`** — currently the `yourname/iconsmaker`
   placeholder. Set the real slug.
2. **Homebrew tap** — create `ideocentric/homebrew-tap` and add the rendered
   `iconsmaker.rb` (attached to each Release), so users can
   `brew tap ideocentric/tap && brew install iconsmaker`.
3. **winget publisher** — the manifests default to the `ideocentric` publisher
   and `ideocentric.iconsmaker` identifier. Adjust if needed, then submit the
   rendered 3-file set (attached to each Release) to
   [`microsoft/winget-pkgs`](https://github.com/microsoft/winget-pkgs).

The `@PLACEHOLDER@` tokens in the Homebrew and winget files are filled in by the
release job — do not hand-edit the rendered outputs.

## Building packages locally

```bash
cargo build --release
mkdir -p target/assets
./target/release/iconsmaker --print-man-page > target/assets/iconsmaker.1

# Debian
cargo install cargo-deb && cargo deb --no-build          # -> target/debian/*.deb

# RPM
cargo install cargo-generate-rpm && cargo generate-rpm   # -> target/generate-rpm/*.rpm

# Windows MSI (on Windows, with the WiX toolset installed)
cargo install cargo-wix && cargo wix
```

## macOS — Homebrew

The tap lives in the separate repo
[`ideocentric/homebrew-tap`](https://github.com/ideocentric/homebrew-tap)
(`Formula/iconsmaker.rb`, a universal-binary install):

```bash
brew tap ideocentric/tap
brew install iconsmaker
```

**Auto-update on release:** the `release` job renders `iconsmaker.rb` (from
`homebrew/iconsmaker.rb.tmpl`) and commits it into the tap on each release.
One-time setup — add a repository **Actions secret** **`HOMEBREW_TAP_TOKEN`**: a
Personal Access Token (fine-grained, scoped to `ideocentric/homebrew-tap` with
**Contents: read and write**) — `GITHUB_TOKEN` can't push to another repo.
Without the secret the step logs a skip, so releases still succeed.

## Fedora COPR

`copr/iconsmaker.spec` builds iconsmaker **from source** (from the GitHub release
tag) — this is what COPR consumes, distinct from the prebuilt `.rpm` the release
CI attaches.

1. Create a COPR project (needs a Fedora/FAS account) at
   <https://copr.fedorainfracloud.org/>, e.g. `ideocentric/iconsmaker`.
2. In the project settings, turn **ON "Enable network for builds"** — the build
   runs `cargo build`, which fetches crates from crates.io.
3. Add a build from `copr/iconsmaker.spec` (or an SRPM built from it). COPR builds
   one RPM per selected chroot/arch.

Users then:

```bash
sudo dnf copr enable ideocentric/iconsmaker
sudo dnf install iconsmaker
```

Bump `Version` and add a `%changelog` entry in the spec for each new release.

### Auto-rebuild on release

The `copr` job in `.github/workflows/release.yml` triggers a COPR build of the
exact tag (via `copr-cli buildscm … --enable-net on`) after each successful
release. One-time setup — add a repository **Actions secret** named
**`COPR_CONFIG`** whose value is your COPR API token block from
<https://copr.fedorainfracloud.org/api/> (the `[copr-cli]` section with
`login`/`username`/`token`/`copr_url`). Without the secret the job just logs a
skip, so releases still succeed. The job assumes the COPR project is
`ideocentric/iconsmaker`.

## Ubuntu PPA

`debian/` holds a vendored, offline-building source package for a Launchpad PPA.
Because Launchpad builds have no network and use the distro `rustc`, there are two
gotchas (crate vendoring, and a `rustc >= 1.85` requirement) — see
[`debian/README.md`](debian/README.md) for the full build/upload flow. Users then:

```bash
sudo add-apt-repository ppa:ideocentric/iconsmaker
sudo apt update && sudo apt install iconsmaker
```

## Windows — winget

The three `winget/ideocentric.iconsmaker.*.yaml` manifests are rendered per
release (x64 + arm64 MSIs, real sha256) and attached to the GitHub Release.
Submit them to [`microsoft/winget-pkgs`](https://github.com/microsoft/winget-pkgs)
with [`wingetcreate`](https://github.com/microsoft/winget-create):

```powershell
# Fetch the rendered manifests from the release, then:
wingetcreate submit --token <gh-token> path\to\manifests\
```

(Or fork winget-pkgs, drop the three files under
`manifests/i/ideocentric/iconsmaker/<version>/`, and open a PR.) Once merged:
`winget install ideocentric.iconsmaker`. A future automation option is the
[winget-releaser](https://github.com/vedantmgoyal9/winget-releaser) action.

## Windows — Scoop

Manifest lives in the separate bucket repo
[`ideocentric/scoop-bucket`](https://github.com/ideocentric/scoop-bucket)
(`bucket/iconsmaker.json`, x64 + arm64, with `checkver`/`autoupdate`):

```powershell
scoop bucket add ideocentric https://github.com/ideocentric/scoop-bucket
scoop install iconsmaker
```

## Arch — AUR

`aur/` holds a source-build `PKGBUILD` + `.SRCINFO`. Arch's rolling `rustc` is
always current, so it builds from the release tag with `cargo`. See
[`aur/README.md`](aur/README.md) for how to push it to the AUR (needs an AUR
account + SSH key). Once published: `yay -S iconsmaker`.