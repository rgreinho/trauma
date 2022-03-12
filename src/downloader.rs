//! Represents the download controller.

use crate::{download::Download, Error};
use futures::stream::{self, StreamExt};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::TracingMiddleware;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tracing::debug;

const DEFAULT_RETRIES: u32 = 3;
const DEFAULT_CONCURRENT_DOWNLOADS: usize = 32;

/// Represents the download controller.
#[derive(Debug, Clone)]
pub struct Downloader {
    /// Directory where to store the downloaded files.
    directory: PathBuf,
    /// Number of retries per downloaded file.
    retries: u32,
    /// Number of maximum concurrent downloads.
    concurrent_downloads: usize,
}

impl Downloader {
    /// Starts the downloads.
    pub async fn download(&self, downloads: &[Download]) {
        // Prepare the HTTP client.
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(self.retries);
        let client = ClientBuilder::new(reqwest::Client::new())
            .with(TracingMiddleware)
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        // Download the files asynchronously.
        let _tasks = stream::iter(downloads)
            .map(|d| self.fetch(&client, d))
            .buffer_unordered(self.concurrent_downloads)
            .collect::<Vec<_>>()
            .await;
    }

    /// Fetches the files and write them to disk.
    async fn fetch(&self, client: &ClientWithMiddleware, download: &Download) -> Result<(), Error> {
        // Request the file.
        debug!("Fetching {}", download.url.clone());
        let res = client.get(download.url.clone()).send().await.unwrap();

        // Prepare the destination directory/file.
        fs::create_dir_all(&self.directory)?;
        let output = self.directory.join(&download.filename);
        debug!("Creating file {:?}", &output);
        let mut file = File::create(output)?;

        // Download the file chunk by chunk.
        debug!("Retrieving chunks...");
        let mut stream = res.bytes_stream();
        while let Some(item) = stream.next().await {
            let chunk = item?;
            file.write_all(&chunk)?;
        }

        Ok(())
    }
}

/// A builder used to create a [`Downloader`].
pub struct DownloaderBuilder(Downloader);

impl DownloaderBuilder {
    /// Creates a builder with the default options.
    pub fn new() -> Self {
        DownloaderBuilder::default()
    }

    /// Sets the directory where to store the [`Download`]s.
    pub fn directory(mut self, directory: PathBuf) -> Self {
        self.0.directory = directory;
        self
    }

    /// Set the number of retries per [`Download`].
    pub fn retries(mut self, retries: u32) -> Self {
        self.0.retries = retries;
        self
    }

    /// Set the number of concurrent [`Download`]s.
    pub fn concurrent_downloads(mut self, concurrent_downloads: usize) -> Self {
        self.0.concurrent_downloads = concurrent_downloads;
        self
    }

    /// Create the [`Downloader`] with the specified options.
    pub fn build(self) -> Downloader {
        Downloader {
            directory: self.0.directory,
            retries: self.0.retries,
            concurrent_downloads: self.0.concurrent_downloads,
        }
    }
}

impl Default for DownloaderBuilder {
    fn default() -> Self {
        Self(Downloader {
            directory: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("")),
            retries: DEFAULT_RETRIES,
            concurrent_downloads: DEFAULT_CONCURRENT_DOWNLOADS,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_builder_defaults() {
        let d = DownloaderBuilder::new().build();
        assert_eq!(d.retries, DEFAULT_RETRIES);
        assert_eq!(d.concurrent_downloads, DEFAULT_CONCURRENT_DOWNLOADS);
    }
}
