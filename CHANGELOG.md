# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
[1.1.0]: https://github.com/rgreinho/trauma/releases/tag/1.1.0
[#16]: https://github.com/rgreinho/trauma/pull/16
[#24]: https://github.com/rgreinho/trauma/pull/24
