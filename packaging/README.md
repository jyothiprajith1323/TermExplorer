# Linux packaging

The repository packaging targets are:

| Format | Distribution family | Installer |
|---|---|---|
| `.deb` | Debian, Ubuntu, Linux Mint, Pop!_OS and compatible derivatives | `sudo apt install ./termexplorer_VERSION_amd64.deb` |
| `.rpm` | Fedora, RHEL-compatible distributions, openSUSE and compatible RPM distributions | `sudo dnf install ./termexplorer-VERSION-1.x86_64.rpm` or `sudo zypper install ./...rpm` |
| `PKGBUILD` | Arch Linux, EndeavourOS, Manjaro and compatible derivatives | `makepkg -si` in `packaging/aur` |
| `.tar.gz` | Other compatible glibc-based Linux environments | Extract and install the binary into a user-local `bin` directory |

## Build Debian and RPM packages

Build the release executable first, then run this script on the target architecture:

```bash
cargo build --locked --release
sudo apt install dpkg-dev rpm
VERSION="$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)"
scripts/package_linux.sh target/release/termexplorer "$VERSION" dist
scripts/test_package_linux.sh "$VERSION" dist
```

The packager requires `dpkg-deb` and `rpmbuild`. RPM runtime requirements are derived by the RPM ELF dependency generator rather than hard-coding distribution-specific package names. The package smoke-test script additionally requires `rpm` and validates package metadata, pre-release ordering, package contents and isolated extraction/installation. The scripts support `x86_64` and `aarch64` hosts and write checksums for generated packages. Pre-release hyphens are converted to Debian/RPM tilde versions (for example, `1.1.0-rc2` becomes `1.1.0~rc2`) so the release candidate sorts before the corresponding final release in package managers.

## Installation notes

- A local `.deb` can be installed by APT, which resolves dependencies declared in the package metadata: `sudo apt install ./termexplorer_VERSION_amd64.deb`. This is not a signed remote APT repository.
- These artifacts are Linux packages, not a promise of universal binary compatibility. They are built for the architecture and ABI shown in the artifact name. The initial release workflow targets Linux x86_64; additional architectures require their own build and clean-install validation.
- Linux Mint, Pop!_OS and other Debian-derived distributions can generally consume the `.deb`; RPM-compatible distributions can generally consume the `.rpm`. Verify each distribution's supported dependency names and ABI before advertising it as tested.
- The portable archive is intended for compatible glibc Linux systems. Alpine Linux/musl is not covered by this build unless a separately tested musl target is added.
- A signed APT repository hosted on GitHub Pages requires signed repository metadata and a managed signing key. Do not add an unsigned `apt` source or embed a private signing key in the repository.
- The Arch `PKGBUILD` is a repository build recipe. It is not evidence that a package has been submitted to or accepted by the AUR.
- Optional tools such as Git, FFmpeg, Poppler and trash utilities are not bundled. Their availability affects only the associated integrations.
- Package artifacts should be published only after the tagged release workflow successfully builds and validates them.
