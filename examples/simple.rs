//! Simple download example.
//!
//! Run with
//!
//! ```not_rust
//! cargo run -q --example simple
//! ```

use std::path::PathBuf;
use trauma::{download::Download, downloader::Downloader, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let url = "https://github.com/seanmonstar/reqwest/archive/refs/tags/v0.11.9.zip";
    let downloads = vec![Download::builder().url(url)?.build()];
    let downloader = Downloader::builder()
        .directory(PathBuf::from("output"))
        .build();
    downloader.download(&downloads).await;
    Ok(())
}
