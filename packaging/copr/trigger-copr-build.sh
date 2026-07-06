#!/usr/bin/env bash
# Trigger a Fedora COPR SCM build of iconsmaker for a given git tag.
#
#   packaging/copr/trigger-copr-build.sh [TAG]     # default: v<version from Cargo.toml>
#
# Why a script? The release CI's `copr` job is fragile — copr-cli 2.5 imports
# `rich` but doesn't declare it, so pipx's ISOLATED venv crashes with
# ModuleNotFoundError: rich — and COPR itself is occasionally flaky, so re-triggers
# are a recurring need. This gives a reliable, repeatable local trigger.
#
# The fix for the rich bug is simply to install copr-cli AND rich into the SAME
# venv (so rich is importable), rather than relying on pipx isolation. This script
# creates a local ./.copr-venv, submits an SCM build from the GitHub tag using the
# packaged spec, and enables network access (the build vendors crates from
# crates.io). It does NOT wait for the build (prints the build URL and exits).
#
# One-time prerequisite: your COPR API token at ~/.config/copr. Get it from
# https://copr.fedorainfracloud.org/api/ — copy the [copr-cli] block into that file
# (chmod 600). The same token block is stored as the repo secret COPR_CONFIG.
set -euo pipefail

REPO_ROOT="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"

TAG="${1:-}"
if [ -z "$TAG" ]; then
  VER="$(sed -n 's/^version = "\([^"]*\)".*/\1/p' "$REPO_ROOT/Cargo.toml" | head -1)"
  [ -n "$VER" ] || { echo "could not read version from Cargo.toml; pass a TAG explicitly"; exit 1; }
  TAG="v${VER}"
fi

if [ ! -f "$HOME/.config/copr" ]; then
  echo "ERROR: COPR API token not found at ~/.config/copr" >&2
  echo "  Get it from https://copr.fedorainfracloud.org/api/ — copy the [copr-cli]" >&2
  echo "  block into ~/.config/copr (chmod 600), then re-run. (Same block as the" >&2
  echo "  repo Actions secret COPR_CONFIG.)" >&2
  exit 1
fi

# copr-cli in a local venv, with rich installed ALONGSIDE it so the CLI can import
# rich (this is the whole workaround for the copr-cli 2.5 missing-dependency bug).
VENV="$REPO_ROOT/.copr-venv"
COPR="$VENV/bin/copr-cli"
if [ ! -x "$COPR" ]; then
  echo ">> Setting up copr-cli (+rich) in $VENV …"
  python3 -m venv "$VENV"
  "$VENV/bin/pip" install --quiet --upgrade pip
  "$VENV/bin/pip" install --quiet copr-cli rich
fi
echo ">> using $("$COPR" --version 2>&1 | head -1)"

echo ">> Submitting COPR SCM build for tag $TAG (spec: packaging/copr/iconsmaker.spec)…"
"$COPR" buildscm ideocentric/iconsmaker \
  --clone-url https://github.com/ideocentric/iconsmaker \
  --commit "$TAG" \
  --spec packaging/copr/iconsmaker.spec \
  --type git --enable-net on --nowait

echo
echo ">> Submitted. Watch the build at:"
echo "   https://copr.fedorainfracloud.org/coprs/ideocentric/iconsmaker/builds/"