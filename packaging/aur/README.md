# AUR Packaging

Scaffolding for publishing StriVo to the Arch User Repository.

## First-time submission

1. Tag a release and let the `Release` workflow attach the tarball + `SHA256SUMS`
   to the GitHub release.
2. Update `PKGBUILD` and `.SRCINFO` in this directory:
   - bump `pkgver`
   - replace `sha256sums=('SKIP')` with the real `sha256sum` of the release
     tarball from the GH release asset
3. Regenerate `.SRCINFO`:
   ```
   cd packaging/aur
   makepkg --printsrcinfo > .SRCINFO
   ```
4. Lint:
   ```
   namcap PKGBUILD
   ```
5. Test build in a clean chroot (recommended):
   ```
   extra-x86_64-build
   ```
   Or a quick local build:
   ```
   makepkg -si
   ```
6. Publish to AUR:
   ```
   git clone ssh://aur@aur.archlinux.org/strivo.git aur-strivo
   cp PKGBUILD .SRCINFO aur-strivo/
   cd aur-strivo
   git add PKGBUILD .SRCINFO
   git commit -m "Initial import 0.3.0"
   git push
   ```

## Updating on new releases

Repeat steps 1–3, commit, push. That's it.
