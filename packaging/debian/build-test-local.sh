#!/usr/bin/env bash
# Fast LOCAL reproduction of the Launchpad PPA build, to catch build failures in
# minutes instead of ~30-45 min per Launchpad round-trip.
#
#   packaging/debian/build-test-local.sh [SERIES]      # default: noble
#
# It mirrors the Launchpad builder as closely as is practical on a dev box:
#   * a clean ubuntu:24.04 (noble) container — arm64-native on Apple Silicon;
#   * Build-Depends installed FROM APT via mk-build-deps, so a wrong cargo-/rustc-
#     version pin in debian/control fails here exactly as on Launchpad (NO rustup);
#   * the full dpkg-source 3.0 (quilt) round-trip (build the source package, then
#     extract it) so vendored-file quirks — e.g. the dropped Cargo.toml.orig —
#     reproduce faithfully;
#   * an offline `dpkg-buildpackage -b` binary build, which is what actually
#     compiles the crate on the builder.
#
# It does NOT sign or upload — run it BEFORE docker-ppa-upload.sh to de-risk.
# The pinned Rust version is read straight from debian/control so this test can
# never drift from what we upload.
#
# To also test the emulated foreign arch (slower): DOCKER_PLATFORM=linux/amd64 …
set -euo pipefail

SERIES="${1:-noble}"
REPO_ROOT="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
PLATFORM_ARG=()
[ -n "${DOCKER_PLATFORM:-}" ] && PLATFORM_ARG=(--platform "$DOCKER_PLATFORM")

command -v docker >/dev/null || { echo "docker not found / not running"; exit 1; }

# Map the Ubuntu series to its Docker base image. Keep in sync with the same map
# in docker-ppa-upload.sh; extend both as new series are targeted.
case "$SERIES" in
  noble)    IMAGE=ubuntu:24.04 ;;   # 24.04 LTS
  resolute) IMAGE=ubuntu:26.04 ;;   # 26.04 LTS
  questing) IMAGE=ubuntu:25.10 ;;
  *) echo "unknown series '$SERIES' — add it to the series->image map in $0"; exit 1 ;;
esac

# Keep the toolchain the test installs in lockstep with the packaging.
RUST_VER="$(sed -n 's/.*rustc-\([0-9.]*\).*/\1/p' "$REPO_ROOT/packaging/debian/control" | head -1)"
[ -n "$RUST_VER" ] || { echo "could not read rustc-<ver> pin from debian/control"; exit 1; }

RUN=/tmp/iconsmaker-build-test.sh
cat > "$RUN" <<'CEOF'
#!/usr/bin/env bash
set -euo pipefail
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y -qq dpkg-dev debhelper devscripts fakeroot equivs \
  git ca-certificates curl >/dev/null

PKG=iconsmaker
VER="$(sed -n '1s/^[^(]*(\([0-9.]*\)-.*/\1/p' /src/packaging/debian/changelog)"
B=/tmp/b; S="$B/$PKG-$VER"; mkdir -p "$S"
git config --global --add safe.directory /src
# Test the WORKING TREE (all tracked files at their current content), NOT HEAD, so
# uncommitted packaging edits are validated before you commit/upload. (docker-ppa-
# upload.sh uploads committed HEAD, so: edit -> build-test -> commit -> upload.)
# Caveat: brand-new files must be `git add`ed to be seen (ls-files = tracked only).
( cd /src && git ls-files -z | tar --null -T - -cf - ) | tar -x -C "$S"

# Overlay the packaging as debian/ and install its Build-Depends from apt exactly
# as declared — this is where a bad cargo-/rustc- pin blows up, just like the PPA.
cp -r "$S/packaging/debian" "$S/debian"
rm -f "$S/debian/build-source-package.sh" "$S/debian/docker-ppa-upload.sh" \
      "$S/debian/build-test-local.sh" "$S/debian/README.md"
chmod +x "$S/debian/rules"
echo ">> Installing declared Build-Depends from apt (mk-build-deps)…"
( cd "$S" && mk-build-deps --install --remove \
    --tool 'apt-get --no-install-recommends -y' debian/control >/dev/null )

# Vendor with the SAME toolchain the build will use (from apt, not rustup).
export PATH="/usr/lib/rust-${RUST_VER}/bin:$PATH"
echo ">> cargo $(cargo --version) / rustc $(rustc --version) — vendoring…"
( cd "$S" && cargo vendor --locked vendor >/dev/null )

# Build the source package, then EXTRACT it: the quilt round-trip is what drops
# vendor/*/Cargo.toml.orig, so building the extracted tree reproduces the builder.
tar -C "$B" -czf "$B/${PKG}_${VER}.orig.tar.gz" "$PKG-$VER"
( cd "$S" && dpkg-source -b . )
rm -rf "$B/x"; mkdir -p "$B/x"
dpkg-source -x "$B/${PKG}_${VER}"-*.dsc "$B/x/$PKG-$VER"

echo ">> Binary build (offline) in the extracted tree — the real test…"
( cd "$B/x/$PKG-$VER" && dpkg-buildpackage -b -uc -us )

echo
echo ">> SUCCESS — produced:"
ls -la "$B/x"/*.deb
echo ">> .deb contents:"
dpkg-deb -c "$B/x"/*.deb
CEOF

echo ">> Local Launchpad-build reproduction (series=$SERIES, image=$IMAGE, rust=$RUST_VER, host arch=$(uname -m)${DOCKER_PLATFORM:+, platform=$DOCKER_PLATFORM})"
docker run --rm ${PLATFORM_ARG[@]+"${PLATFORM_ARG[@]}"} \
  -e RUST_VER="$RUST_VER" \
  -v "$REPO_ROOT:/src:ro" \
  -v "$RUN:/build-test.sh:ro" \
  "$IMAGE" bash /build-test.sh