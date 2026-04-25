//! Demonstrates per-download progress bar labels using `Download.tag`
//! and the `display_tag` option on the downloader.
//!
//! Run with:
//!
//! ```not_rust
//! cargo run -q --example with-tag
//! ```

use std::path::PathBuf;
use trauma::{
    download::Download,
    downloader::{DownloaderBuilder, StyleOptions},
    Error,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let url_a =
        "https://github.com/lxl66566/GalgameManager/releases/download/v1.1.1/GalgameManager_1.1.1_x64_en-US.msi";
    let url_b = "https://github.com/lxl66566/GalgameManager/releases/download/v1.1.1/GalgameManager_1.1.1_amd64.AppImage";

    let default_label = Download::try_from(url_a).unwrap();
    let custom_label = Download::try_from(url_b)
        .unwrap()
        .with_tag("Galgame Manager");

    // display_tag = true
    //
    // Tags are left-aligned to the width of the longest one so bars line up:
    //
    // GalgameManager_1.1.1_x64_en-US.msi ━━╾─────────────────────────────────────    1.05 MiB/14.28 MiB
    // Galgame Manager                    ━━━╴────────────────────────────────────    1.10 MiB/14.28 MiB
    //
    // If no tag is set, the filename is used as a fallback.
    println!("=== Pass A: display_tag = true ===\n");
    let downloader = DownloaderBuilder::new()
        .directory(PathBuf::from("output"))
        .style_options(StyleOptions::default())
        .build();
    downloader
        .download(&[default_label.clone(), custom_label.clone()])
        .await;

    // display_tag = false
    //
    // No tag prefix is prepended
    println!("\n=== Pass B: display_tag = false ===\n");
    let downloader = DownloaderBuilder::new()
        .directory(PathBuf::from("output"))
        .style_options(StyleOptions::default())
        .display_tag(false)
        .build();
    downloader.download(&[default_label, custom_label]).await;

    Ok(())
}
