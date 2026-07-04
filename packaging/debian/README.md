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

2. **The builder's default `rustc` lags upstream.** iconsmaker is edition 2024,
   which needs **rustc ≥ 1.85**, but Ubuntu's *unversioned* `rustc` trails it
   (noble 24.04 ships 1.75). Depending on `rustc (>= 1.85)` is therefore
   unsatisfiable on the LTS and the build fails at `install-deps` with
   `sbuild-build-depends-main-dummy : Depends: rustc (>= 1.85)`.
   The fix (already in place): Build-Depend on a **co-installable versioned
   toolchain** and prepend its `/usr/lib/rust-<ver>/bin` to `PATH` in
   `debian/rules` so the plain `cargo`/`rustc` calls resolve to it. We currently
   pin **`cargo-1.89` / `rustc-1.89`** — not because edition 2024 needs it, but
   because a transitive dependency (`image` 0.25.x) requires **rustc ≥ 1.88**,
   and noble-updates skips 1.86–1.88 (it jumps 1.85 → 1.89 → 1.91). Both 1.89
   and 1.91 are published for amd64 **and** arm64. To move the pin, bump the
   version in **`control` and `rules` in lockstep**. If the dependency tree's
   MSRV climbs again, cargo will name the offending crate and its required
   rustc — pin to the next available versioned toolchain at or above it.

## One-time setup

- A [Launchpad account](https://launchpad.net/) and a PPA named `iconsmaker`.
- A GPG key **registered with Launchpad** (its email must match the changelog /
  the key you sign with).
- On a Linux box: `sudo apt install devscripts debhelper dpkg-dev` and a Rust
  toolchain ≥ 1.85 (`rustup` works), plus `dput`.

## Build & upload

### On a Linux box

From a clean checkout at the release tag:

```bash
packaging/debian/build-source-package.sh noble     # or another recent series
dput ppa:ideocentric/iconsmaker /tmp/…/iconsmaker_<ver>-1~ppa1~noble1_source.changes
```

### From macOS (or any host without Debian tooling) — via Docker

```bash
packaging/debian/docker-ppa-upload.sh noble        # run in your own terminal
```

Run it in a **real terminal** (it's interactive — GPG prompts for your key
passphrase inside the container). It exports your signing key into an ephemeral
`--rm` container (temp dir deleted on exit), vendors the crates, builds + signs
the source package, and `dput`s it. The source-package build has been validated
in an `ubuntu:24.04` container; only the signing/upload is untested (needs your
key + a TTY).

For multiple series, re-run either script with each series name — it appends
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