# Ubuntu PPA packaging

Debian source packaging for publishing iconsmaker to a
[Launchpad PPA](https://launchpad.net/~ideocentric/+archive/ubuntu/iconsmaker).
Users then:

```bash
sudo add-apt-repository ppa:ideocentric/iconsmaker
sudo apt update
sudo apt install iconsmaker
```

## Two Rust-on-Launchpad gotchas (important)

1. **Builds are network-isolated.** Launchpad's build farm has no internet, so
   `cargo` cannot fetch from crates.io. Dependencies are therefore **vendored**
   into `vendor/` (via `cargo vendor`) and the build runs `--offline`. The
   `build-source-package.sh` helper does the vendoring for you; `debian/rules`
   points cargo at the vendored sources.

2. **The builder uses the distro `rustc`.** iconsmaker is edition 2024, which
   needs **rustc ≥ 1.85** (`debian/control` Build-Depends on `rustc (>= 1.85)`).
   The PPA will only build on Ubuntu series whose `rustc` meets that — i.e.
   recent series. Older series (e.g. jammy) ship an older rustc and will fail the
   build-dependency. Options: target only recent series, or configure a Rust
   toolchain PPA as a dependency in the Launchpad project's build settings.

## One-time setup

- A [Launchpad account](https://launchpad.net/) and a PPA named `iconsmaker`.
- A GPG key **registered with Launchpad** (its email must match the changelog /
  the key you sign with).
- On a Linux box: `sudo apt install devscripts debhelper dpkg-dev` and a Rust
  toolchain ≥ 1.85 (`rustup` works), plus `dput`.

## Build & upload

From a clean checkout at the release tag:

```bash
packaging/debian/build-source-package.sh noble     # or another recent series
dput ppa:ideocentric/iconsmaker /tmp/…/iconsmaker_0.1.0-1~ppa1~noble1_source.changes
```

For multiple series, re-run with each series name — the helper appends
`~<series>1` to the version so each upload is unique in the PPA.

## Files

| File | Purpose |
|---|---|
| `control` | Source/binary metadata, Build-Depends (`cargo`, `rustc >= 1.85`) |
| `rules` | debhelper rules; offline cargo build + man page install |
| `changelog` | Version + target series (retargeted by the helper) |
| `copyright` | DEP-5; GPL-3.0-or-later + a simplified `vendor/*` stanza |
| `source/format` | `3.0 (quilt)` |
| `build-source-package.sh` | Vendors deps, builds the orig tarball, runs `debuild -S` |

> **Untested locally** — none of this can be exercised on macOS (no
> `dpkg-buildpackage`). Expect to iterate once on a Linux box / the first
> Launchpad build (the vendored-offline build and the rustc-version constraint
> are the two things most likely to need a tweak).