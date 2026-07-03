# AUR package

A source-build [AUR](https://aur.archlinux.org/) package for iconsmaker. Arch is
rolling, so its `rustc` is always current (≥ 1.85) — the package builds cleanly
from the GitHub release tag with `cargo`. Users then:

```bash
yay -S iconsmaker        # or: paru -S iconsmaker
```

## Publishing / updating (needs an AUR account + SSH key registered with the AUR)

The AUR is one git repo per package. To publish:

```bash
git clone ssh://aur@aur.archlinux.org/iconsmaker.git
cd iconsmaker
cp /path/to/iconsmaker/packaging/aur/{PKGBUILD,.SRCINFO} .

# Best practice: build/test in a clean chroot and regenerate .SRCINFO locally
makepkg -si                 # build + install to sanity-check
makepkg --printsrcinfo > .SRCINFO

git add PKGBUILD .SRCINFO
git commit -m "iconsmaker 0.1.0-1"
git push
```

For each new release: bump `pkgver` (reset `pkgrel=1`), update the `sha256sums`
entry (sha256 of `.../archive/refs/tags/v<ver>.tar.gz`), regenerate `.SRCINFO`,
and push.

> The committed `.SRCINFO` here is hand-written to match `PKGBUILD`; regenerate it
> with `makepkg --printsrcinfo` on an Arch box before pushing to be safe.