//! Showcases the resume feature.
//!
//! Run with
//!
//! ```not_rust
//! cargo run -q --example with-resume
//! ```

use color_eyre::{
    eyre::{eyre, Report},
    Result,
};
use futures::stream::StreamExt;
use rand::Rng;
use reqwest::{
    header::{ACCEPT_RANGES, RANGE},
    Url,
};
use std::{fs, path::PathBuf};
use tokio::{fs::File, io::AsyncWriteExt};
use tracing::debug;
use trauma::{download::Download, downloader::DownloaderBuilder};

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

    // We must ensure that the download is resumable to prove our point.
    assert!(resumable);

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
    let mut file = File::create(&output).await?;
    while let Some(item) = stream.next().await {
        file.write_all_buf(&mut item?).await?;
    }
    debug!("Retrieved {} bytes.", random_bytes);

    // Download the rest of the bits with the [`Downloader`].
    let dl = Download::new(
        &avatar,
        output
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or(eyre!("invalid path terminator"))?,
    );
    let downloads = vec![dl];

    // Hidding the progress bar because of the logging.
    let downloader = DownloaderBuilder::hidden()
        .directory(output.parent().unwrap().to_path_buf())
        .build();
    downloader.download(&downloads).await;

    Ok(())
}
