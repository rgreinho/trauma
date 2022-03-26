//! Simple download example.
//!
//! Run with
//!
//! ```not_rust
//! cargo run -q --example simple
//! ```

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
