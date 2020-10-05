# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## Changed

- Encountering an error spawning the Voxygen process no longer crashes Airshipper [#94](https://github.com/Songtronix/Airshipper/pull/94)
- Fallback to terminal incase the GUI fails [#97](https://github.com/Songtronix/Airshipper/pull/97)

## [0.4.0] - 2020-08-29

## Added

- bundle ssl for better cross distro compatibility [#67](https://github.com/Songtronix/Airshipper/pull/67)
- embed assets for easier distribution [#67](https://github.com/Songtronix/Airshipper/pull/67)
- use same font as Veloren [#67](https://github.com/Songtronix/Airshipper/pull/67)
- option to retry if download/install failed [#78](https://github.com/Songtronix/Airshipper/pull/78)
- offline support [#87](https://github.com/Songtronix/Airshipper/pull/87)

## Changed

- place screenshots, logs, game saves in profile [#74](https://github.com/Songtronix/Airshipper/pull/74)
- do not close Veloren if airshipper gets closed [#77](https://github.com/Songtronix/Airshipper/pull/77)
- prune log file automatically [87](https://github.com/Songtronix/Airshipper/pull/87)

## Removed

- extra terminal window [#67](https://github.com/Songtronix/Airshipper/pull/67)

## [0.3.2] - 2020-03-21

## Changed

- fix: update news when remote is newer [#40](https://github.com/Songtronix/Airshipper/pull/40)
- prefer dedicated gpu [#41](https://github.com/Songtronix/Airshipper/pull/41)
- log crashes in file [#45](https://github.com/Songtronix/Airshipper/pull/45)
- updated all dependencies [#46](https://github.com/Songtronix/Airshipper/pull/46)
- fix: avoid unneeded warning about corrupted version [#47](https://github.com/Songtronix/Airshipper/pull/47)

## [0.3.1] - 2020-03-09

## Changed

- Fixes critical issues for new users trying out airshipper. [#38](https://github.com/Songtronix/Airshipper/issues/38)

## [0.3.0] - 2020-02-23

### Added

- Airshipper updates itself on windows [#9](https://github.com/Songtronix/Airshipper/issues/9)
- Changelog [#26](https://github.com/Songtronix/Airshipper/issues/26)

### Changed

- remove outdated assets automatically [#28](https://github.com/Songtronix/Airshipper/issues/28)
- Updated iced to latest version [#31](https://github.com/Songtronix/Airshipper/issues/31)
- include platform in request header [#34](https://github.com/Songtronix/Airshipper/issues/34)

## [0.2.1] - 2020-02-06

### Changed

- fixed `VCRUNTIME140_1.dll` missing [#16](https://github.com/Songtronix/Airshipper/issues/16)
- Linux builds are zips but file ending is .tar.gz [#14](https://github.com/Songtronix/Airshipper/issues/14)
- airshipper does not display changelog and news [#13](https://github.com/Songtronix/Airshipper/issues/13)
- made GUI optional [#12](https://github.com/Songtronix/Airshipper/issues/12)

## [0.2.0] - 2020-02-02

### Added

- Added GUI

[unreleased]: https://github.com/Songtronix/Airshipper/compare/v0.4.0...master
[0.2.1]: https://github.com/Songtronix/Airshipper/releases/tag/v0.2.1
[0.2.0]: https://github.com/Songtronix/Airshipper/releases/tag/v0.2.0
[0.3.0]: https://github.com/Songtronix/Airshipper/releases/tag/v0.3.0
[0.3.1]: https://github.com/Songtronix/Airshipper/releases/tag/v0.3.1
[0.3.2]: https://github.com/Songtronix/Airshipper/releases/tag/v0.3.2
[0.4.0]: https://github.com/Songtronix/Airshipper/releases/tag/v0.4.0
