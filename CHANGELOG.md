# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Handled 416 status when a file is already fully downloaded. [#86]

[#86]: https://github.com/rgreinho/trauma/pull/86

## [2.2.4]

### Fixed

- Fixed a silent failure when a download filename contains a path. [#73]

[#73]: https://github.com/rgreinho/trauma/pull/73
[2.2.4]: https://github.com/rgreinho/trauma/releases/tag/2.2.4

## [2.2.3] - 2023-05-14

### Fixed

- Ensured that custom headers are used for all the downloader requests, for
  instance, propagating authentication headers to check whether a download can
  be resumed. [#62]

[#62]: https://github.com/rgreinho/trauma/pull/62
[2.2.3]: https://github.com/rgreinho/trauma/releases/tag/2.2.3

## [2.2.2] - 2023-04-26

### Fixed

- Fixed the total size which was incorrectly reported in the progress bar. [#59]

[#59]: https://github.com/rgreinho/trauma/pull/59
[2.2.2]: https://github.com/rgreinho/trauma/releases/tag/2.2.2

## [2.2.1] - 2023-03-06

### Fixed

- Handled the case when the server does not send a `content-length` header,
  causing downloads to be skipped. [#56]

[#56]: https://github.com/rgreinho/trauma/pull/56
[2.2.1]: https://github.com/rgreinho/trauma/releases/tag/2.2.1

## [2.2.0] - 2022-01-21

### Added

- Added the ability to provide custom HTTP headers to the downloader. [#53]

[#53]: https://github.com/rgreinho/trauma/pull/53
[2.2.0]: https://github.com/rgreinho/trauma/releases/tag/2.2.0

## [2.1.1] - 2022-11-19

### Fixed

- Fixed a bug preventing the progress bars to be hidden. [#45]

### Changed

- Upgraded [indicatif] from 0.17.0-rc.10 to 0.17.2. [#45]

[#45]: https://github.com/rgreinho/trauma/pull/45
[2.1.1]: https://github.com/rgreinho/trauma/releases/tag/2.1.1

## [2.1.0] - 2022-09-10

### Added

- Added the ability to use a proxy. [#33]

### Fixed

- Fixed the filename parsing when constructing from URL. [#33]

[#33]: https://github.com/rgreinho/trauma/pull/33
[2.1.0]: https://github.com/rgreinho/trauma/releases/tag/2.1.0

## [2.0.0] - 2022-04-21

### Added

- Added the ability to resume downloads. [#26]

### Changed

- Removed the `skip_existing` option. [#26]

### Fixed

- Fixed a bug preventing the progress bars to be disabled. [#29]

[#26]: https://github.com/rgreinho/trauma/pull/26
[#29]: https://github.com/rgreinho/trauma/pull/29
[2.0.0]: https://github.com/rgreinho/trauma/releases/tag/2.0.0

## [1.1.0] - 2022-04-15

### Added

- Added ability to skip a download if a file with the same name exists at the
  destination. [#16]
- Added ability to customize the progress bars [#24]
  - Customize the format
  - Customize the progression style
  - Leave them on the screen or clear them upon completion
  - Hide any or both of them
  - Add preconfigured styles

[#16]: https://github.com/rgreinho/trauma/pull/16
[#24]: https://github.com/rgreinho/trauma/pull/24
[1.1.0]: https://github.com/rgreinho/trauma/releases/tag/1.1.0

## [1.0.0] - 2022-03-29

Initial version with the following feature set:

- Library only
- HTTP(S) downloads
- Download files via providing a list of URLs
  - Ability to rename downloaded files
- Ability to configure the download manager
  - Download directory
  - Maximum simultaneous requests
  - Number of retries
- Asynchronous w/ [Tokio]
- Progress bar w/ [trauma]
  - Display the individual progress
  - Display the total progress

[1.0.0]: https://github.com/rgreinho/trauma/releases/tag/1.0.0
