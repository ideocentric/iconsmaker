#!/usr/bin/env bash
# Build a signed Ubuntu *source* package for the Launchpad PPA.
#
# Run on a LINUX box that has: a Rust toolchain >= 1.85 (rustup is fine),
# cargo, devscripts, debhelper, dpkg-dev, and a GPG key registered on Launchpad.
# Run it from a clean checkout of the iconsmaker repo at the tag you're releasing.
#
#   packaging/debian/build-source-package.sh [SERIES]   # default: noble
#
# It exports a clean upstream tree, vendors all crates (network needed HERE, not
# on Launchpad), builds the orig tarball (upstream + vendor), lays down debian/,
# retargets the changelog to SERIES, and runs `debuild -S`. Upload the resulting
# *_source.changes with `dput`.
set -euo pipefail

VERSION="0.1.0"
PKG="iconsmaker"
SERIES="${1:-noble}"

REPO_ROOT="$(git rev-parse --show-toplevel)"
BUILD_DIR="$(mktemp -d)"
SRC="${BUILD_DIR}/${PKG}-${VERSION}"

echo ">> Exporting clean upstream tree -> ${SRC}"
mkdir -p "${SRC}"
git -C "${REPO_ROOT}" archive --format=tar HEAD | tar -x -C "${SRC}"

echo ">> Vendoring crates (requires network)"
( cd "${SRC}" && cargo vendor --locked vendor >/dev/null )

echo ">> Building orig tarball (upstream + vendor, no debian/)"
tar -C "${BUILD_DIR}" -czf "${BUILD_DIR}/${PKG}_${VERSION}.orig.tar.gz" "${PKG}-${VERSION}"

echo ">> Laying down debian/ and retargeting changelog to '${SERIES}'"
cp -r "${REPO_ROOT}/packaging/debian" "${SRC}/debian"
rm -f "${SRC}/debian/build-source-package.sh" "${SRC}/debian/README.md"
chmod +x "${SRC}/debian/rules"
# Distribution -> SERIES, and make the version unique per series for the PPA.
sed -i "1s/) [a-z]*;/) ${SERIES};/" "${SRC}/debian/changelog"
sed -i "1s/(\\([^)]*\\))/(\\1~${SERIES}1)/" "${SRC}/debian/changelog"

echo ">> Building signed source package (debuild -S -sa)"
( cd "${SRC}" && debuild -S -sa )

echo
echo ">> Artifacts in ${BUILD_DIR}:"
ls -1 "${BUILD_DIR}"/*.changes "${BUILD_DIR}"/*.dsc 2>/dev/null || true
echo
echo ">> Upload with:"
echo "     dput ppa:ideocentric/${PKG} ${BUILD_DIR}/${PKG}_${VERSION}-1~ppa1~${SERIES}1_source.changes"