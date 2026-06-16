//! Download example using a target filename with a path.
//!
//! Run with
//!
//! ```not_rust
//! cargo run -q --example with-path-as-filename
//! ```
use trauma::{download::Download, downloader::Downloader, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let url = "https://github.com/seanmonstar/reqwest/archive/refs/tags/v0.11.9.zip";
    let downloads = vec![Download::builder()
        .url(url)?
        .filename_override("output/test_dir/reqwest.zip")
        .build()];
    let downloader = Downloader::builder().build();
    downloader.download(&downloads).await;
    Ok(())
}
