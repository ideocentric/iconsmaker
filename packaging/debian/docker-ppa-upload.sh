#!/usr/bin/env bash
# Build, sign, and upload the iconsmaker source package to the Launchpad PPA using
# Docker on macOS (or any host without Debian tooling).
#
#   packaging/debian/docker-ppa-upload.sh [SERIES]      # default: noble
#
# RUN THIS IN YOUR OWN TERMINAL — it is interactive: GPG will prompt for your
# signing-key passphrase inside the container (needs a TTY; it can't be driven
# headlessly). For multiple Ubuntu series, run it once per series name.
#
# What it does: exports your signing key into an ephemeral `--rm` container (the
# temp export dir is deleted on exit), then inside that container it vendors the
# crates, assembles the source package, signs it with your key, and `dput`s it.
#
# Prerequisites:
#   * Docker running.
#   * The oss@ideocentric.com secret key in your host GnuPG.
#   * The ppa:ideocentric/iconsmaker PPA created, key registered, CoC signed.
#   * Target a series that carries the versioned Rust toolchain the packaging
#     pins (cargo-1.91/rustc-1.91) — noble (24.04) and resolute (26.04) both do,
#     for amd64+arm64. Add new series to the series->image map below. See
#     packaging/debian/README.md for how the rustc-version constraint is handled.
set -euo pipefail

SERIES="${1:-noble}"
KEY="E45E7C1B2DD9EE19653E4B2E4508404C0CEFDF94"   # oss@ideocentric.com signing key
REPO_ROOT="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# Map the Ubuntu series to its Docker base image. Keep in sync with the same map
# in build-test-local.sh; extend both as new series are targeted.
case "$SERIES" in
  noble)    IMAGE=ubuntu:24.04 ;;   # 24.04 LTS
  resolute) IMAGE=ubuntu:26.04 ;;   # 26.04 LTS
  questing) IMAGE=ubuntu:25.10 ;;
  *) echo "unknown series '$SERIES' — add it to the series->image map in $0"; exit 1 ;;
esac

command -v docker >/dev/null || { echo "docker not found / not running"; exit 1; }

echo ">> Exporting signing key (your host GPG may prompt for the passphrase)…"
gpg --export-secret-keys --armor "$KEY" > "$WORK/secret.asc"

# Container-side steps (kept in a file to avoid quoting pain).
cat > "$WORK/run.sh" <<'CEOF'
#!/usr/bin/env bash
set -euo pipefail
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y -qq dpkg-dev debhelper devscripts fakeroot dput \
  curl git ca-certificates gnupg pinentry-curses jq >/dev/null
# rustup stable — the distro cargo can be too old to parse edition 2024 for `cargo vendor`
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
  | sh -s -- -y --profile minimal --default-toolchain stable >/dev/null 2>&1
. "$HOME/.cargo/env"

# GPG: import the signing key; use a TTY pinentry so it can prompt for the passphrase.
mkdir -p ~/.gnupg && chmod 700 ~/.gnupg
echo "pinentry-program /usr/bin/pinentry-curses" > ~/.gnupg/gpg-agent.conf
gpgconf --kill gpg-agent >/dev/null 2>&1 || true
export GPG_TTY=$(tty)
gpg --batch --import /work/secret.asc

PKG=iconsmaker
VER="$(sed -n '1s/^[^(]*(\([0-9.]*\)-.*/\1/p' /src/packaging/debian/changelog)"
ORIG="${PKG}_${VER}.orig.tar.gz"
B=/tmp/b; S="$B/$PKG-$VER"; mkdir -p "$S"   # $S needed by the first-upload tar; the reuse path just repopulates it
git config --global --add safe.directory /src

# The orig tarball is named by the UPSTREAM version only ($VER), independent of
# the ~ppaN/~series revision. Launchpad rejects a re-upload whose orig has the
# same name but different bytes ("already exists ... but ... different contents").
# So if an orig for $VER already exists in the PPA, download and REUSE it verbatim
# (checksum matches) and upload source-only (-sd). Otherwise build it (-sa).
#
# Check BOTH Published AND Pending: when bumping several series at once, the first
# series' source may still be Pending when the next runs, but its orig is already
# in the archive — so a Published-only check would miss it, re-vendor a different
# orig, and get rejected. (This is exactly what bit the resolute 0.1.2 upload.)
PPA_API="https://api.launchpad.net/devel/~ideocentric/+archive/ubuntu/iconsmaker"
echo ">> Checking whether $ORIG already exists in the PPA (Published or Pending)…"
ORIG_URL="$(for st in Published Pending; do
    curl -sSL "${PPA_API}?ws.op=getPublishedSources&status=${st}" | jq -r '.entries[].self_link'
  done \
  | while read -r sp; do curl -sSL "${sp}?ws.op=sourceFileUrls" | jq -r '.[]'; done \
  | grep -F "/${ORIG}" | head -1 || true)"

if [ -n "$ORIG_URL" ]; then
  echo ">> Reusing already-published orig (source-only upload): $ORIG_URL"
  curl -sSL "$ORIG_URL" -o "$B/$ORIG"
  tar -xzf "$B/$ORIG" -C "$B"            # recreates $B/$PKG-$VER (source + vendor)
  SA_FLAG=-sd
else
  echo ">> No published orig for $VER; creating one (first upload)…"
  git -C /src archive --format=tar HEAD | tar -x -C "$S"
  echo ">> Vendoring crates…"
  ( cd "$S" && cargo vendor --locked vendor >/dev/null )
  tar -C "$B" -czf "$B/$ORIG" "$PKG-$VER"
  SA_FLAG=-sa
fi

# Overlay the CURRENT packaging (committed HEAD) as debian/ — this is what carries
# our fixes; the reused orig deliberately keeps the old upstream payload.
rm -rf "$S/debian"
cp -r /src/packaging/debian "$S/debian"
rm -f "$S/debian/build-source-package.sh" "$S/debian/docker-ppa-upload.sh" \
      "$S/debian/build-test-local.sh" "$S/debian/README.md"
chmod +x "$S/debian/rules"
# Retarget the changelog to $SERIES and make the version unique per series.
sed -i "1s/) [a-z]*;/) ${SERIES};/" "$S/debian/changelog"
sed -i "1s/(\\([^)]*\\))/(\\1~${SERIES}1)/" "$S/debian/changelog"

echo ">> Building + signing (enter your GPG passphrase when prompted)…"
# -d: skip the build-dependency check. Build-Depends (cargo-1.89, rustc-1.89) are
# satisfied on Launchpad's builder, not here — we only used rustup to vendor.
# $SA_FLAG: -sa on first upload (include orig), -sd on re-upload (reuse archived orig).
( cd "$S" && debuild -S "$SA_FLAG" -d -k"$KEY" )

echo ">> Uploading to ppa:ideocentric/iconsmaker…"
dput ppa:ideocentric/iconsmaker "$B"/${PKG}_*_source.changes
CEOF

docker run --rm -it \
  -e SERIES="$SERIES" -e KEY="$KEY" \
  -v "$REPO_ROOT:/src:ro" \
  -v "$WORK:/work" \
  "$IMAGE" bash /work/run.sh

echo
echo ">> Done. Watch the build at:"
echo "   https://launchpad.net/~ideocentric/+archive/ubuntu/iconsmaker"