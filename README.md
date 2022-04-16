# Trauma

[![Crates.io](https://img.shields.io/crates/v/trauma.svg)](https://crates.io/crates/trauma)
[![Documentation](https://docs.rs/trauma/badge.svg)](https://docs.rs/trauma/)
[![ci](https://github.com/rgreinho/trauma/actions/workflows/ci-rust.yml/badge.svg)](https://github.com/rgreinho/trauma/actions/workflows/ci-rust.yml)

Tokio Rust Asynchronous Universal download MAnager

## Description

`Trauma is a library simplifying and prettifying HTTP(s) downloads. The
downloads are executed asynchronously and progress bars are drawn on the screen
to help monitoring the process.

![screenshot](assets/pip-style.png)

### Features

- Library only
- HTTP(S) downloads
- Download files via providing a list of URLs
  - Ability to rename downloaded files
- Ability to configure the download manager
  - Download directory
  - Maximum simultaneous requests
  - Number of retries
- Asynchronous w/ [Tokio]
- Progress bar w/ [indicatif]
  - Display the individual progress
  - Display the total progress
- Ability to customize the progress bars
  - Customize the format
  - Customize the progression style
  - Leave them on the screen or clear them upon completion
  - Hide any or both of them
  - Add pre-configured styles

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
trauma = "1.0"
```

## Quick start

```rust
use std::path::PathBuf;
use trauma::{download::Download, downloader::DownloaderBuilder, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let reqwest_rs = "https://github.com/seanmonstar/reqwest/archive/refs/tags/v0.11.9.zip";
    let downloads = vec![Download::try_from(reqwest_rs).unwrap()];
    let downloader = DownloaderBuilder::new()
        .directory(PathBuf::from("output"))
        .build();
    downloader.download(&downloads).await;
    Ok(())
}
```

More examples can be found in the [examples](examples) folder.

## Why another download manager

Before starting this project, I spent some time searching the internet, trying
not to reinvent the wheel. And I did find a bunch of interesting existing
projects!

However they are almost all abandoned:

- DLM: <https://github.com/agourlay/dlm>
  - Active, but is just a binary/CLI tool
- Snatch: <https://github.com/derniercri/snatch>
  - Inactive since Sept '17
  - Recommend switching to [Zou]
- Zou: <https://github.com/k0pernicus/zou>
  - Inactive since Oct '17
- Duma: <https://github.com/mattgathu/duma>
  - Inactive since Nov '20
- Siwi: <https://github.com/rs-videos/siwi-download>
  - Inactive since Mar '21
- Downloader: <https://github.com/hunger/downloader>
  - Dying project
  - No answers to issues/PRs
  - Only automated updates are being merged
  - No release since Feb '21

As a result, I decided to write `trauma`.

### Potential ideas for future versions

- Resume download
- Support for other download protocol (i.e.: FTP)

[indicatif]: https://github.com/console-rs/indicatif
[tokio]: https://tokio.rs/
[zou]: https://github.com/k0pernicus/zou
