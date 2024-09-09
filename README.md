# Airshipper

[![GitHub release)](https://img.shields.io/github/v/release/veloren/airshipper?include_prereleases)](https://github.com/veloren/Airshipper/releases) [![License](https://img.shields.io/github/license/veloren/airshipper?color=blue)](https://github.com/veloren/Airshipper/blob/master/LICENSE) [![Discord](https://img.shields.io/discord/449602562165833758?label=discord)](https://discord.gg/rvbW3Z4) [![AUR version](https://img.shields.io/aur/version/airshipper?label=AUR)](https://aur.archlinux.org/packages/airshipper/)

A cross-platform Veloren launcher.

![Airshipper](https://i.imgur.com/1VkndRZ.gif)

## Features

- [x] Update/Download and start nightly/weekly.
- [x] Fancy UI with batteries included.
- [x] Updates itself on windows.

## Download

**NOTE:** Airshipper cannot be considered stable yet.

For *binary* packages the gitlab releases can be used.

For *source* packages **do not** use the `master` branch. Always package latest release either via tag (`v*.*.*`) or branch (`r*.*`) as master is unstable and contains work in progress features.

#### Compile from source

```bash
git clone https://gitlab.com/veloren/airshipper.git
cd airshipper
cargo run --release
```

Make sure to have [rustup](https://rustup.rs/) installed to compile rust code and [git lfs](https://book.veloren.net/contributors/development-tools.html#git-lfs) for assets.

### Airshipper-Server

**NOTE:** Airshipper-Server is not required by end-users.

#### Compile from source

```bash
git clone https://gitlab.com/veloren/airshipper.git
cd airshipper
cargo run --release --bin airshipper-server
```

On first execution, a template configuration file will be created at `config/config.template.ron` and the server will exit.

Rename this to `config.ron` and edit as appropriate before running again.

```bash
cargo run --release --bin airshipper-server
```

#### For NixOS users

You can install Airshipper with:
- Flakes enabled Nix: `nix profile install gitlab:veloren/Airshipper`
- Flakes disabled Nix: `nix-env -i -f "https://gitlab.com/veloren/Airshipper/tarball/master"`
