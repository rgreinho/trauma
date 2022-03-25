//! Represents the download controller.

use crate::{download::Download, download::Status, download::Summary};
use futures::stream::{self, StreamExt};
use http::StatusCode;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::TracingMiddleware;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
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
    pub async fn download(&self, downloads: &[Download]) -> Vec<Summary> {
        // Prepare the HTTP client.
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(self.retries);
        let client = ClientBuilder::new(reqwest::Client::new())
            .with(TracingMiddleware)
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        // Prepare the progress bar.
        let multi = Arc::new(MultiProgress::new());
        let style = ProgressStyle::with_template("{bar:40.green/yellow} {pos:>7}/{len:7}").unwrap();
        let main =
            Arc::new(multi.add(ProgressBar::new(downloads.len() as u64).with_style(style.clone())));
        main.tick();

        // Download the files asynchronously.
        stream::iter(downloads)
            .map(|d| self.fetch(&client, d, multi.clone(), main.clone()))
            .buffer_unordered(self.concurrent_downloads)
            .collect::<Vec<_>>()
            .await
    }

    /// Fetches the files and write them to disk.
    async fn fetch(
        &self,
        client: &ClientWithMiddleware,
        download: &Download,
        multi: Arc<MultiProgress>,
        main: Arc<ProgressBar>,
    ) -> Summary {
        // Create a download summary.
        let mut summary = Summary::new(download.clone(), StatusCode::BAD_REQUEST, 0);

        // Request the file.
        debug!("Fetching {}", &download.url);
        let res = match client.get(download.url.clone()).send().await {
            Ok(res) => res,
            Err(e) => {
                return summary.fail(e);
            }
        };

        // Check the status for errors.
        match res.error_for_status_ref() {
            Ok(_res) => (),
            Err(e) => {
                return summary.fail(e);
            }
        };

        // Update the summary with the collected details.
        let size = res.content_length().unwrap_or_default();
        let status = res.status();
        summary = Summary::new(download.clone(), status, size);

        // Create the progress bar.
        let style = ProgressStyle::with_template(
            "{bar:40.cyan/blue} {bytes}/{total_bytes} ({bytes_per_sec}, {eta})",
        )
        .unwrap();
        let pb = multi.add(ProgressBar::new(size).with_style(style.clone()));

        // Prepare the destination directory/file.
        match fs::create_dir_all(&self.directory) {
            Ok(_res) => (),
            Err(e) => {
                return summary.fail(e);
            }
        };
        let output = self.directory.join(&download.filename);
        debug!("Creating file {:?}", &output);
        let mut file = match File::create(output) {
            Ok(file) => file,
            Err(e) => {
                return summary.fail(e);
            }
        };

        // Download the file chunk by chunk.
        debug!("Retrieving chunks...");
        let mut stream = res.bytes_stream();
        while let Some(item) = stream.next().await {
            // Retrieve chunk.
            let chunk = match item {
                Ok(chunk) => chunk,
                Err(e) => {
                    return summary.fail(e);
                }
            };
            pb.inc(chunk.len() as u64);

            // Write the chunk to disk.
            match file.write_all(&chunk) {
                Ok(_res) => (),
                Err(e) => {
                    return summary.fail(e);
                }
            };
        }

        // Remove the bar once complete.
        pb.finish_and_clear();

        // Advance the main progress bar.
        main.inc(1);

        // Return the download summary.
        summary.with_status(Status::Success)
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
            directory: std::env::current_dir().unwrap_or_default(),
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
