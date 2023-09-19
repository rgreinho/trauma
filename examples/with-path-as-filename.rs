//! Download example using a target filename with a path.
//!
//! Run with
//!
//! ```not_rust
//! cargo run -q --example with-path-as-filename
//! ```
use trauma::{download::Download, downloader::DownloaderBuilder, Error};

use reqwest::Url;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let reqwest_rs = "https://github.com/seanmonstar/reqwest/archive/refs/tags/v0.11.9.zip";
    let downloads = vec![Download {
        url: Url::parse(reqwest_rs).unwrap(),
        filename: "output/test_dir/reqwest.zip".to_string(),
    }];
    let downloader = DownloaderBuilder::new().build();
    downloader.download(&downloads).await;
    Ok(())
}
