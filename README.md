# ZFS Toolbox

A native GTK4 desktop application to manage OpenZFS on Linux: pools, datasets,
snapshots and retention, replication (send/receive), disk health and scrubs,
mounting and native encryption keys.

Available in English, French, German and Spanish. The interface follows the
system language and falls back to English for any other language.

> **Status:** early development. Version 0.1 contains the foundations: the
> application shows the OpenZFS version and the imported pools, and checks
> administrator access. The other sections arrive in the next versions.

## Supported systems

| Family | Minimum version | Package |
| --- | --- | --- |
| Debian, Ubuntu and derivatives | Debian 12, Ubuntu 22.04 | `.deb` |
| Red Hat, Rocky, Alma and derivatives | 9 (with EPEL) | `.rpm` |
| Any recent distribution | glibc 2.35 | AppImage |

OpenZFS itself must be installed (`zfsutils-linux` on Debian and Ubuntu,
`zfs` on Red Hat). GTK 4.6 and libadwaita 1.1 are the minimum versions.

The AppImage contains the application only. Administrator actions need the
helper and its polkit rule, which are installed by the `.deb` and `.rpm`
packages.

## Architecture

```
crates/
  zfs-core/            Pure library: runs `zfs`/`zpool` in script mode
                       (-H -p), parses their output, validates names and
                       defines the helper protocol. Unit-tested.
  zfs-toolbox/         The GTK4 + libadwaita application. Runs as the user.
  zfs-toolbox-helper/  Privileged helper, started through pkexec.
data/                  .desktop, AppStream metainfo, polkit rule, icon.
po/                    Translations (gettext).
packaging/             AppImage build script.
```

Reading the state of ZFS does not need administrator rights. Every change goes
through `zfs-toolbox-helper`, started with `pkexec`: polkit asks for an
administrator password (action `io.github.gillesmagneville.ZfsToolbox.manage`,
remembered for a few minutes). The application sends one JSON request on the
helper's standard input. The helper only knows a fixed list of operations,
validates every name and runs `zfs`/`zpool` without a shell.

## Building

Dependencies on Debian and Ubuntu:

```sh
sudo apt install build-essential pkg-config libgtk-4-dev libadwaita-1-dev gettext
```

On Red Hat 9 and derivatives:

```sh
sudo dnf install epel-release && sudo dnf config-manager --set-enabled crb
sudo dnf install gcc pkgconf-pkg-config gtk4-devel libadwaita-devel gettext
```

Then install Rust with [rustup](https://rustup.rs) and run:

```sh
cargo build --release --workspace
scripts/i18n.sh build                    # translations and data files
ZFS_TOOLBOX_LOCALEDIR=target/data/locale \
ZFS_TOOLBOX_HELPER=$PWD/target/release/zfs-toolbox-helper \
  target/release/zfs-toolbox
```

Packages (also built by the CI on every push):

```sh
cargo deb -p zfs-toolbox --no-build             # needs cargo-deb
cargo generate-rpm -p crates/zfs-toolbox        # needs cargo-generate-rpm
packaging/appimage/build.sh
```

Pushing a `v*` tag publishes the three packages as a GitHub release.

## Translations

Source strings are in English, in the code and in `data/*.in`.

- `scripts/i18n.sh update` extracts the strings into `po/zfs-toolbox.pot` and
  merges them into `po/fr.po`, `po/de.po` and `po/es.po`.
- `scripts/i18n.sh check` fails when a string is missing or fuzzy in any
  language. The CI runs it.

In Rust, keep translatable strings on a single line, in `gettext("…")`,
`ngettext("…", "…", n)` or `gettext_f("… {} …", &[…])`.

To add a language, add its code to `po/LINGUAS`, run
`scripts/i18n.sh update`, translate the new `po/<code>.po` and add the `.mo`
file to the package lists in `crates/zfs-toolbox/Cargo.toml`.

## License

ZFS Toolbox is free software, released under the
[GNU General Public License v3.0 or later](LICENSE).
