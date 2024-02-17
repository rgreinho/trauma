//! Range error handling example.
//!
//! Setup for this example:
//!
//! From the root of the project:
//! ```not_rust
//! mkdir -p examples/miniserve
//! cd examples/miniserve
//! curl -sLO http://212.183.159.230/5MB.zip
//! miniserve .
//! ```
//! Run with:
//!
//! ```not_rust
//! cargo run -q --example range-error-handling
//! ```
//!
//! Miniserve is a utility written in rust to serve files over HTTP:
//! https://github.com/svenstaro/miniserve

use std::path::PathBuf;
use trauma::{download::Download, downloader::DownloaderBuilder, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let five_mb = "http://localhost:8080/5MB.zip";
    let downloads = vec![Download::try_from(five_mb).unwrap()];
    let downloader = DownloaderBuilder::new()
        .directory(PathBuf::from("output"))
        .build();
    let summary = downloader.download(&downloads).await;
    dbg!(summary);
    Ok(())
}
