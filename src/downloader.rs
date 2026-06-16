//! Represents the download controller.

use crate::{
    download::{Download, Status, Summary},
    ResponseExt,
};
use bon::Builder;
use futures::stream::{self, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use reqwest::header::{HeaderMap, RANGE};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::TracingMiddleware;
use std::{path::PathBuf, sync::Arc};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};
use tracing::debug;

const DEFAULT_RETRIES: u32 = 3;
const DEFAULT_CONCURRENT_DOWNLOADS: usize = 32;

/// Represents the download controller.
///
/// A downloader can be created via its builder:
///
/// ```rust
/// # fn main()  {
/// use trauma::downloader::Downloader;
///
/// let d = Downloader::builder().build();
/// # }
/// ```
#[derive(Debug, Clone, Builder)]
pub struct Downloader {
    /// Directory where to store the downloaded files.
    #[builder(default = std::env::current_dir().unwrap_or_default(), into )]
    directory: PathBuf,
    /// Number of retries per downloaded file.
    #[builder(default = DEFAULT_RETRIES)]
    retries: u32,
    /// Number of maximum concurrent downloads.
    #[builder(default = DEFAULT_CONCURRENT_DOWNLOADS)]
    concurrent_downloads: usize,
    /// Downloader style options.
    #[builder(default)]
    style_options: StyleOptions,
    /// Resume the download if necessary and possible.
    #[builder(default = true)]
    resumable: bool,
    /// Custom HTTP headers.
    headers: Option<HeaderMap>,
}

impl Default for Downloader {
    fn default() -> Self {
        Downloader::builder().build()
    }
}

impl Downloader {
    const ALREADY_DOWNLOADED: &str = "the file was already fully downloaded";

    /// Starts the downloads.
    pub async fn download(&self, downloads: &[Download]) -> Vec<Summary> {
        self.download_inner(downloads, None).await
    }

    /// Starts the downloads with proxy.
    pub async fn download_with_proxy(
        &self,
        downloads: &[Download],
        proxy: reqwest::Proxy,
    ) -> Vec<Summary> {
        self.download_inner(downloads, Some(proxy)).await
    }

    /// Starts the downloads.
    async fn download_inner(
        &self,
        downloads: &[Download],
        proxy: Option<reqwest::Proxy>,
    ) -> Vec<Summary> {
        // Prepare the HTTP client.
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(self.retries);

        let mut inner_client_builder = reqwest::Client::builder();
        if let Some(proxy) = proxy {
            inner_client_builder = inner_client_builder.proxy(proxy);
        }
        if let Some(headers) = &self.headers {
            inner_client_builder = inner_client_builder.default_headers(headers.clone());
        }
        let inner_client = inner_client_builder
            .build()
            .expect("the inner client to build");

        let client = ClientBuilder::new(inner_client)
            // Trace HTTP requests. See the tracing crate to make use of these traces.
            .with(TracingMiddleware::default())
            // Retry failed requests.
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        // Prepare the progress bar.
        let multi = match self.style_options.is_hidden() {
            true => Arc::new(MultiProgress::with_draw_target(ProgressDrawTarget::hidden())),
            false => Arc::new(MultiProgress::new()),
        };
        let main = Arc::new(
            multi.add(
                self.style_options
                    .main
                    .clone()
                    .to_progress_bar(downloads.len() as u64),
            ),
        );
        main.tick();

        // Download the files asynchronously.
        let summaries = stream::iter(downloads)
            .map(|d| self.fetch(&client, d, multi.clone(), main.clone()))
            .buffer_unordered(self.concurrent_downloads)
            .collect::<Vec<_>>()
            .await;

        // Finish the progress bar.
        if self.style_options.main.clear {
            main.finish_and_clear();
        } else {
            main.finish();
        }

        // Return the download summaries.
        summaries
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
        let summary = Summary::builder().download(download.clone());

        // Retrieve download metadata.
        let response = match download.head(client).await {
            Ok(r) => r,
            Err(e) => {
                return summary.status(Status::Fail(e.to_string())).build();
            }
        };

        // Try to build the output path.
        let Some(filename) = download.infer_filename(&response) else {
            return summary
                .status(Status::Fail(
                    "Cannot extract the filename. Verify the URL and/or provide an override."
                        .to_string(),
                ))
                .build();
        };
        let output = self.directory.join(&filename);
        debug!("Filename: {filename}");

        // Check if there is a file on disk already.
        let size_on_disk: u64 = match tokio::fs::metadata(&output).await {
            Ok(m) => {
                debug!("A file with the same name already exists at the destination.");
                // If so, check file length to know where to restart the download from.
                m.len()
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => 0,
            Err(e) => {
                return summary.status(Status::Fail(e.to_string())).build();
            }
        };

        // Determine whether the download is resumable or not.
        let can_resume = self.resumable && Download::is_resumable(&response);

        // Update the summary accordingly.
        let summary = summary.resumable(can_resume);

        // Only appends to the file when resumable is set and we can resume.
        let can_append = self.resumable && can_resume;

        // Check whether or not we need to download the file.
        let content_length_value = response.content_length_header().unwrap_or_default();
        if size_on_disk > 0 && size_on_disk == content_length_value {
            return summary
                .status(Status::Skipped(Self::ALREADY_DOWNLOADED.into()))
                .build();
        }

        // Build the GET request.
        debug!("Fetching {}", &download.url_as_str());
        let mut req = client.get(download.url_as_str());

        // If resumable is turned on, request the remaining bytes.
        if self.resumable && can_resume {
            req = req.header(RANGE, format!("bytes={size_on_disk}-"));
        }

        // Add extra headers if needed.
        if let Some(ref h) = self.headers {
            req = req.headers(h.to_owned());
        }

        // Ensure there was no error while sending the request.
        let res = match req.send().await {
            Ok(res) => res,
            Err(e) => {
                return summary.status(Status::Fail(e.to_string())).build();
            }
        };

        // Check the status for errors.
        match res.error_for_status_ref() {
            Ok(_res) => (),
            Err(e) => return summary.status(Status::Fail(e.to_string())).build(),
        };

        // Update the size with the value from the response headers from the
        // GET request, if possible.
        let size = res.content_length_header().unwrap_or_default();

        // Update the summary with the collected details.
        let summary = summary.statuscode(res.status());

        // If there is nothing else to download for this file, we can return.
        if size_on_disk > 0 && size_on_disk == size {
            return summary
                .size(size)
                .status(Status::Skipped(Self::ALREADY_DOWNLOADED.into()))
                .build();
        }

        // Create the child progress bar.
        // If the download is being resumed, the progress bar position is
        // updated to start where the download stopped before.
        let child_opts = self.style_options.child.clone();

        // Add the child progress bar to the main one.
        let pb = multi.add(
            child_opts
                .to_progress_bar(size)
                .with_position(size_on_disk)
                .with_message(filename),
        );

        // Prepare the destination directory.
        let output_dir = output.parent().unwrap_or(&output);
        debug!("Creating destination directory {:?}", output_dir);
        if let Err(e) = tokio::fs::create_dir_all(output_dir).await {
            return summary
                .size(size)
                .status(Status::Fail(e.to_string()))
                .build();
        }

        // Prepare the destination file.
        debug!("Creating destination file {:?}", &output);
        let mut file = match OpenOptions::new()
            .create(true)
            .write(true)
            .append(can_append)
            .truncate(!can_append)
            .open(output)
            .await
        {
            Ok(file) => file,
            Err(e) => {
                return summary
                    .size(size)
                    .status(Status::Fail(e.to_string()))
                    .build();
            }
        };

        // Prepare the final size.
        // We will add the amount of bytes downloaded to the amount of bytes
        // that are already on disk.
        let mut final_size = size_on_disk;

        // Download the file chunk by chunk.
        debug!("Retrieving chunks...");
        let mut stream = res.bytes_stream();
        while let Some(item) = stream.next().await {
            // Retrieve chunk.
            let mut chunk = match item {
                Ok(chunk) => chunk,
                Err(e) => {
                    return summary
                        .size(final_size)
                        .status(Status::Fail(e.to_string()))
                        .build();
                }
            };
            let chunk_size = chunk.len() as u64;
            final_size += chunk_size;
            pb.inc(chunk_size);

            // Write the chunk to disk.
            match file.write_all_buf(&mut chunk).await {
                Ok(_res) => (),
                Err(e) => {
                    return summary
                        .size(final_size)
                        .status(Status::Fail(e.to_string()))
                        .build();
                }
            };
        }

        // Finish the progress bar once complete, and optionally remove it.
        if self.style_options.child.clear {
            pb.finish_and_clear();
        } else {
            pb.finish();
        }

        // Advance the main progress bar.
        main.inc(1);

        // Return a successful summary with the actual download size.
        summary.size(final_size).status(Status::Success).build()
    }
}

/// Define the [`Downloader`] options.
///
/// By default, the main progress bar will stay on the screen upon completion,
/// but the child ones will be cleared once complete.
#[derive(Debug, Clone, Builder)]
pub struct StyleOptions {
    /// Style options for the main progress bar.
    main: ProgressBarOpts,
    /// Style options for the child progress bar(s).
    child: ProgressBarOpts,
}

impl Default for StyleOptions {
    fn default() -> Self {
        Self::builder()
            .main(
                ProgressBarOpts::builder()
                    .template(ProgressBarOpts::TEMPLATE_BAR_WITH_POSITION)
                    .progress_chars(ProgressBarOpts::CHARS_FINE)
                    .build(),
            )
            .child(ProgressBarOpts::with_pip_style())
            .build()
    }
}

impl StyleOptions {
    /// Set the options for the main progress bar.
    pub fn set_main(&mut self, main: ProgressBarOpts) {
        self.main = main;
    }

    /// Set the options for the child progress bar.
    pub fn set_child(&mut self, child: ProgressBarOpts) {
        self.child = child;
    }

    /// Check whether both progress bars are hidden.
    pub fn is_hidden(&self) -> bool {
        self.main.hidden && self.child.hidden
    }

    /// Convenience function to hide the progress bars.
    pub fn hidden() -> Self {
        Self::builder()
            .main(ProgressBarOpts::hidden())
            .child(ProgressBarOpts::hidden())
            .build()
    }
}

/// Define the options for a progress bar.
#[derive(Debug, Clone, Builder)]
#[builder(on(String, into))]
pub struct ProgressBarOpts {
    /// Progress bar template string.
    template: Option<String>,
    /// Progression characters set.
    ///
    /// There must be at least 3 characters for the following states:
    /// "filled", "current", and "to do".
    progress_chars: Option<String>,
    /// Hide the progress bar.
    #[builder(default,  with = || true)]
    hidden: bool,
    /// Clear the progress bar once completed.
    #[builder(default, with = || true)]
    clear: bool,
}

impl Default for ProgressBarOpts {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl ProgressBarOpts {
    /// Template representing the bar and its position.
    ///
    ///`█████████████████████ 11/12 (99%) eta 00:00:02 archive.zip`
    pub const TEMPLATE_BAR_WITH_POSITION: &'static str =
        "{bar:20.blue} {pos:>4}/{len:4} ({percent}%) eta {eta_precise:.blue} {msg}";
    /// Template which looks like the Python package installer pip.
    ///
    /// `━━━━━╾──────────────    3.78 MiB/13.43 MiB    191.39 KiB/s eta 52s  GalgameManager_1.1.1_x64_en-US.msi`
    pub const TEMPLATE_PIP: &'static str =
        "{bar:20.green/black} {bytes:>11.green}/{total_bytes:<11.green} {bytes_per_sec:>13.red} eta {eta:4.blue} {msg}";
    /// Use increasing quarter blocks as progress characters: `"█▛▌▖  "`.
    pub const CHARS_BLOCKY: &'static str = "█▛▌▖  ";
    /// Use fade-in blocks as progress characters: `"█▓▒░  "`.
    pub const CHARS_FADE_IN: &'static str = "█▓▒░  ";
    /// Use fine blocks as progress characters: `"█▉▊▋▌▍▎▏  "`.
    pub const CHARS_FINE: &'static str = "█▉▊▋▌▍▎▏  ";
    /// Use a line as progress characters: `"━╾─"`.
    pub const CHARS_LINE: &'static str = "━╾╴─";
    /// Use rough blocks as progress characters: `"█  "`.
    pub const CHARS_ROUGH: &'static str = "█  ";
    /// Use increasing height blocks as progress characters: `"█▇▆▅▄▃▂▁  "`.
    pub const CHARS_VERTICAL: &'static str = "█▇▆▅▄▃▂▁  ";

    /// Create a [`ProgressStyle`] based on the provided options.
    pub fn to_progress_style(self) -> ProgressStyle {
        let mut style = ProgressStyle::default_bar();
        if let Some(template) = self.template {
            style = style.template(&template).unwrap();
        }
        if let Some(progress_chars) = self.progress_chars {
            style = style.progress_chars(&progress_chars);
        }
        style
    }

    /// Create a [`ProgressBar`] based on the provided options.
    pub fn to_progress_bar(self, len: u64) -> ProgressBar {
        // Return a hidden Progress bar if we hid it.
        if self.hidden {
            return ProgressBar::hidden();
        }

        // Otherwise returns a ProgressBar with the style.
        let style = self.to_progress_style();
        ProgressBar::new(len).with_style(style)
    }

    /// Create a new [`ProgressBarOpts`] which looks like Python pip.
    pub fn with_pip_style() -> Self {
        Self::builder()
            .template(ProgressBarOpts::TEMPLATE_PIP)
            .progress_chars(ProgressBarOpts::CHARS_LINE)
            .clear()
            .build()
    }

    /// Create a new [`ProgressBarOpts`] which hides the progress bars.
    pub fn hidden() -> Self {
        Self::builder().hidden().build()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_builder_defaults() {
        let d = Downloader::builder().build();
        assert_eq!(d.retries, DEFAULT_RETRIES);
        assert_eq!(d.concurrent_downloads, DEFAULT_CONCURRENT_DOWNLOADS);
        assert!(d.resumable);
    }

    #[test]
    fn test_builder_resumable_toggle() {
        let d = Downloader::builder().resumable(false).build();
        assert!(!d.resumable);
    }
}
