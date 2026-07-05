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

### Test the build locally first (strongly recommended)

Launchpad's build farm has a ~30–45 min round-trip, and every failure we hit was
an *environment* mismatch (toolchain version, vendored-file handling, offline
behaviour) — none of which the source-package scripts below exercise, because
they only assemble the source package, they never compile it. So test first:

```bash
packaging/debian/build-test-local.sh noble        # ~few min; needs Docker
```

This reproduces the **builder** (not just the source package) in a clean
`ubuntu:24.04` container: it installs the declared `Build-Depends` **from apt**
via `mk-build-deps` (so a wrong `cargo-`/`rustc-` pin fails here exactly as on
Launchpad — no rustup), runs the full `dpkg-source` 3.0 (quilt) round-trip (so
vendored-file quirks like the dropped `Cargo.toml.orig` reproduce), and does an
**offline `dpkg-buildpackage -b`** binary build. It prints the resulting `.deb`
contents on success. The pinned Rust version is read straight from
`debian/control`, so the test can never drift from what you upload. It's
arm64-native on Apple Silicon; `DOCKER_PLATFORM=linux/amd64 …` also tests the
(emulated, slower) other arch. **Only upload once this produces a `.deb`.**

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
| `control` | Source/binary metadata, Build-Depends (`cargo-1.89`, `rustc-1.89`) |
| `rules` | debhelper rules; offline cargo build (abs vendor path + checksum fixup) + man page install |
| `changelog` | Version + target series (retargeted by the helper) |
| `copyright` | DEP-5; GPL-3.0-or-later + a simplified `vendor/*` stanza |
| `source/format` | `3.0 (quilt)` |
| `build-test-local.sh` | **Reproduces the Launchpad builder in Docker (apt toolchain, quilt round-trip, offline build) — run before uploading** |
| `docker-ppa-upload.sh` | macOS/Docker: reuse orig, build + sign the source package, `dput` to the PPA |
| `build-source-package.sh` | Linux-box helper: vendors deps, builds the orig tarball, runs `debuild -S` |

> **Untested locally** — none of this can be exercised on macOS (no
> `dpkg-buildpackage`). Expect to iterate once on a Linux box / the first
> Launchpad build (the vendored-offline build and the rustc-version constraint
> are the two things most likely to need a tweak).