# Airshipper

[![GitHub release)](https://img.shields.io/github/v/release/veloren/airshipper?include_prereleases)](https://github.com/veloren/Airshipper/releases) [![License](https://img.shields.io/github/license/veloren/airshipper?color=blue)](https://github.com/veloren/Airshipper/blob/master/LICENSE) [![Discord](https://img.shields.io/discord/449602562165833758?label=discord)](https://discord.gg/rvbW3Z4) [![AUR version](https://img.shields.io/aur/version/airshipper?label=AUR)](https://aur.archlinux.org/packages/airshipper/)

A cross-platform Veloren launcher.

![Airshipper](https://camo.githubusercontent.com/71dfc8bb095129c57a7d2c29ff7d50bba4c91e67fef84c2e6ef93be7efb1e02a/68747470733a2f2f7777772e736f6e6774726f6e69782e636f6d2f616972736869707065722d302e342e302e676966)

## Features

- [x] Update/Download and start nightly.
- [x] Fancy UI with batteries included.
- [x] Updates itself on windows.

## Download

**NOTE:** Airshipper cannot be considered stable yet.

#### Compile from source

```bash
git clone https://github.com/veloren/Airshipper.git
cd Airshipper
cargo run --release
```

Make sure to have [rustup](https://rustup.rs/) installed to compile rust code and [git lfs](https://book.veloren.net/contributors/development-tools.html#git-lfs) for assets.

#### For NixOS users

You can install Airshipper with:
- Flakes enabled Nix: `nix profile install github:veloren/Airshipper`
- Flakes disabled Nix: `nix-env -i -f "https://github.com/veloren/Airshipper/tarball/master"`