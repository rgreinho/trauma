//! Showcases the resume feature.
//!
//! Run with
//!
//! ```not_rust
//! cargo run -q --example with-resume
//! ```

use color_eyre::{eyre::Report, Result};
use futures::stream::StreamExt;
use rand::Rng;
use reqwest::header::{ACCEPT_RANGES, RANGE};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tracing_subscriber;
use trauma::{download::Download, downloader::DownloaderBuilder};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Report> {
    // Setup the application.
    color_eyre::install()?;

    // Setup logging.
    tracing_subscriber::fmt::fmt()
        .with_env_filter("with_resume=debug,trauma=debug")
        .init();

    // Prepare the download.
    let avatar = Url::parse("https://avatars.githubusercontent.com/u/6969134?v=4").unwrap();
    let output = PathBuf::from("output/avatar.jpg");
    fs::create_dir_all(output.parent().unwrap())?;

    // Make sure the server accepts range requests.
    let res = reqwest::Client::new()
        .head(&avatar.to_string())
        .send()
        .await?;
    let headers = res.headers();
    let resumable = match headers.get(ACCEPT_RANGES) {
        None => false,
        Some(x) if x == "none" => false,
        Some(_) => true,
    };
    tracing::debug!("Is the file resumable: {:?}", &resumable);

    // If resumable...
    if resumable {
        // Request a random amount of data to simulate a previously failed download.
        let mut rng = rand::thread_rng();
        let random_bytes: u8 = rng.gen();
        let res = reqwest::Client::new()
            .get(&avatar.to_string())
            .header(RANGE, format!("bytes=0-{}", random_bytes))
            .send()
            .await?;

        // Retrieve the bits.
        let mut stream = res.bytes_stream();
        let mut file = File::create(&output)?;
        while let Some(item) = stream.next().await {
            file.write_all(&item?)?;
        }
    }

    // Download the rest of the bits with the [`Downloader`].
    let dl = Download::new(&avatar, output.file_name().unwrap().to_str().unwrap());
    let downloads = vec![dl];
    let downloader = DownloaderBuilder::new()
        .directory(output.parent().unwrap().to_path_buf())
        .build();
    downloader.download(&downloads).await;

    Ok(())
}
