# RPM spec for building iconsmaker from source on Fedora COPR.
#
# COPR setup:
#   1. Create a COPR project (needs a Fedora/FAS account) at
#      https://copr.fedorainfracloud.org/ — e.g. `ideocentric/iconsmaker`.
#   2. IMPORTANT: turn ON "Enable network for builds" in the project settings —
#      the build runs `cargo build`, which fetches crates from crates.io.
#      (Alternative for a network-isolated build: vendor deps with `cargo vendor`,
#       ship them as a second Source, and build with `--offline`.)
#   3. Add a build pointing at this spec (or upload an SRPM). COPR builds one RPM
#      per selected chroot/arch (x86_64, aarch64, ...).
#
# Users then:
#   dnf copr enable ideocentric/iconsmaker
#   dnf install iconsmaker
#
# Bump `Version` (and add a %changelog entry) for each release — do it in the SAME
# commit as the Cargo.toml bump, BEFORE cutting the vX.Y.Z tag. COPR builds the spec
# from whichever committish it clones, but Source0 fetches vX.Y.Z.tar.gz keyed on
# THIS Version, so a stale Version here silently rebuilds the old release. (If the
# tag was already cut without the bump, trigger the build from `main`:
# packaging/copr/trigger-copr-build.sh main)

Name:           iconsmaker
Version:        0.1.2
Release:        1%{?dist}
Summary:        Generate platform icon bundles (macOS, Windows, Linux) from a single SVG

License:        GPL-3.0-or-later
URL:            https://github.com/ideocentric/iconsmaker
Source0:        %{url}/archive/refs/tags/v%{version}.tar.gz#/%{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust >= 1.85

# Pure-Rust build (no system C libraries); the binary links only glibc, so there
# are no explicit runtime Requires.

# Skip debuginfo extraction — simplest reliable path for a cargo-built binary in
# a COPR convenience build.
%global debug_package %{nil}

%description
iconsmaker reads a single source SVG (plus a small TOML config or command-line
flags) and generates deployment-ready icon assets for macOS (.icns), Windows
(.ico) and Linux (freedesktop hicolor PNG tree, .desktop entry, AppStream
metainfo), plus optional Snap, Flatpak and AppImage packaging scaffolding.

%prep
%autosetup -n %{name}-%{version}

%build
# Cargo.lock is committed, so --locked builds the exact pinned dependency set.
cargo build --release --locked

%install
install -Dm0755 target/release/%{name} %{buildroot}%{_bindir}/%{name}
# Generate the man page from the freshly built binary (same arch as the build).
./target/release/%{name} --print-man-page > %{name}.1
install -Dm0644 %{name}.1 %{buildroot}%{_mandir}/man1/%{name}.1

%check
./target/release/%{name} --version

%files
%license LICENSE
%doc README.md
%{_bindir}/%{name}
%{_mandir}/man1/%{name}.1*

%changelog
* Mon Jul 06 2026 Matt Comeione <6080346+ideocentric@users.noreply.github.com> - 0.1.2-1
- New upstream release 0.1.2 (Windows static-CRT fix; no Fedora-visible change)
* Fri Jul 03 2026 Matt Comeione <6080346+ideocentric@users.noreply.github.com> - 0.1.1-1
- Maintenance release (dead-code cleanup; packaging updates)
* Fri Jul 03 2026 Matt Comeione <6080346+ideocentric@users.noreply.github.com> - 0.1.0-1
- Initial package