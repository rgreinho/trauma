//! Represents the download controller.

use crate::download::{Download, Status, Summary};
use futures::stream::{self, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use reqwest::{
    header::{HeaderMap, HeaderValue, IntoHeaderName, RANGE},
    StatusCode,
};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::TracingMiddleware;
use std::{fs, path::PathBuf, sync::Arc};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};
use tracing::debug;

pub struct TimeTrace;

/// Represents the download controller.
///
/// A downloader can be created via its builder:
///
/// ```rust
/// # fn main()  {
/// use trauma::downloader::DownloaderBuilder;
///
/// let d = DownloaderBuilder::new().build();
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Downloader {
    /// Directory where to store the downloaded files.
    directory: PathBuf,
    /// Number of retries per downloaded file.
    retries: u32,
    /// Number of maximum concurrent downloads.
    concurrent_downloads: usize,
    /// Downloader style options.
    style_options: StyleOptions,
    /// Resume the download if necessary and possible.
    resumable: bool,
    headers: Option<HeaderMap>,
}

impl Downloader {
    const DEFAULT_RETRIES: u32 = 3;
    const DEFAULT_CONCURRENT_DOWNLOADS: usize = 32;

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
    pub async fn download_inner(
        &self,
        downloads: &[Download],
        proxy: Option<reqwest::Proxy>,
    ) -> Vec<Summary> {
        // Prepare the HTTP client.
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(self.retries);

        let inner_client = proxy.map_or_else(reqwest::Client::new, |p| {
            reqwest::Client::builder().proxy(p).build().unwrap()
        });

        let client = ClientBuilder::new(inner_client)
            // Trace HTTP requests. See the tracing crate to make use of these traces.
            .with(TracingMiddleware::default())
            // Retry failed requests.
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        // Prepare the progress bar.
        let multi = match self.style_options.clone().is_enabled() {
            true => Arc::new(MultiProgress::new()),
            false => Arc::new(MultiProgress::with_draw_target(ProgressDrawTarget::hidden())),
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
        let mut size_on_disk: u64 = 0;
        let mut can_resume = false;
        let output = self.directory.join(&download.filename);
        let mut summary = Summary::new(
            download.clone(),
            StatusCode::BAD_REQUEST,
            size_on_disk,
            can_resume,
        );

        // If resumable is turned on...
        if self.resumable {
            can_resume = match download.is_resumable(client).await {
                Ok(r) => r,
                Err(e) => {
                    return summary.fail(e);
                }
            };

            // Check if there is a file on disk already.
            if can_resume && output.exists() {
                debug!("A file with the same name already exists at the destination.");
                // If so, check file length to know where to restart the download from.
                size_on_disk = match output.metadata() {
                    Ok(m) => m.len(),
                    Err(e) => {
                        return summary.fail(e);
                    }
                }
            }

            // Update the summary accordingly.
            summary.set_resumable(can_resume);
        }

        // Request the file.
        debug!("Fetching {}", &download.url);
        let mut req = client.get(download.url.clone());
        if self.resumable && can_resume {
            req = req.header(RANGE, format!("bytes={}-", size_on_disk));
        }

        if let Some(ref h) = self.headers {
            req = req.headers(h.to_owned());
        }

        let res = match req.send().await {
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
        summary = Summary::new(download.clone(), status, size, can_resume);

        // If there is nothing else to download for this file, we can return.
        if size_on_disk > 0 && size == size_on_disk {
            return summary.with_status(Status::Skipped(
                "the file was already fully downloaded".into(),
            ));
        }

        // Create the progress bar.
        // If the download is being resumed, the progress bar position is
        // updated to start where the download stopped before.
        let pb = multi.add(
            self.style_options
                .child
                .clone()
                .to_progress_bar(size)
                .with_position(size_on_disk),
        );

        // Prepare the destination directory/file.
        debug!("Creating destination directory {:?}", &self.directory);
        match fs::create_dir_all(&self.directory) {
            Ok(_res) => (),
            Err(e) => {
                return summary.fail(e);
            }
        };

        debug!("Creating destination file {:?}", &output);
        // append: If we can't resume from where we left off,
        //         we should overrwite the file and start again
        //         This also prevents corrupting files by writing
        //         to them again
        // write:  We are writing to the file
        // create: The file should be created if it doesn't exist
        let mut file = match OpenOptions::new()
            .append(can_resume)
            .write(true) 
            .create(true)
            .open(output)
            .await
        {
            Ok(file) => file,
            Err(e) => {
                return summary.fail(e);
            }
        };

        let mut final_size = size_on_disk;

        // Download the file chunk by chunk.
        debug!("Retrieving chunks...");
        let mut stream = res.bytes_stream();
        while let Some(item) = stream.next().await {
            // Retrieve chunk.
            let mut chunk = match item {
                Ok(chunk) => chunk,
                Err(e) => {
                    return summary.fail(e);
                }
            };
            let chunk_size = chunk.len() as u64;
            final_size += chunk_size;
            pb.inc(chunk_size);

            // Write the chunk to disk.
            match file.write_all_buf(&mut chunk).await {
                Ok(_res) => (),
                Err(e) => {
                    return summary.fail(e);
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

        // Create a new summary with the real download size
        println!("Size on disk {}", size_on_disk);
        let summary = Summary::new(download.clone(), status, final_size, can_resume);
        // Return the download summary.
        summary.with_status(Status::Success)
    }
}

/// A builder used to create a [`Downloader`].
///
/// ```rust
/// # fn main()  {
/// use trauma::downloader::DownloaderBuilder;
///
/// let d = DownloaderBuilder::new().retries(5).directory("downloads".into()).build();
/// # }
/// ```
pub struct DownloaderBuilder(Downloader);

impl DownloaderBuilder {
    /// Creates a builder with the default options.
    pub fn new() -> Self {
        DownloaderBuilder::default()
    }

    /// Convenience function to hide the progress bars.
    pub fn hidden() -> Self {
        let d = DownloaderBuilder::default();
        d.style_options(StyleOptions::new(
            ProgressBarOpts::hidden(),
            ProgressBarOpts::hidden(),
        ))
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

    /// Set the downloader style options.
    pub fn style_options(mut self, style_options: StyleOptions) -> Self {
        self.0.style_options = style_options;
        self
    }

    fn new_header(&self) -> HeaderMap {
        match self.0.headers {
            Some(ref h) => h.to_owned(),
            _ => HeaderMap::new(),
        }
    }

    /// Add the http headers.
    ///
    /// You need to pass in a `HeaderMap`, not a `HeaderName`.
    /// `HeaderMap` is a set of http headers.
    ///
    /// You can call `.headers()` multiple times and all `HeaderMap` will be merged into a single one.
    ///
    /// # Example
    ///
    /// ```
    /// use reqwest::header::{self, HeaderValue, HeaderMap};
    /// use trauma::downloader::DownloaderBuilder;
    ///
    /// let ua = HeaderValue::from_str("curl/7.87").expect("Invalid UA");
    ///
    /// let builder = DownloaderBuilder::new()
    ///     .headers(HeaderMap::from_iter([(header::USER_AGENT, ua)]))
    ///     .build();
    /// ```
    ///
    /// See also [`header()`].
    ///
    /// [`header()`]: DownloaderBuilder::header
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        let mut new = self.new_header();
        new.extend(headers);

        self.0.headers = Some(new);
        self
    }

    /// Add the http header
    ///
    /// # Example
    ///
    /// You can use the `.header()` chain to add multiple headers
    ///
    /// ```
    /// use reqwest::header::{self, HeaderValue};
    /// use trauma::downloader::DownloaderBuilder;
    ///
    /// const FIREFOX_UA: &str =
    /// "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/109.0";
    ///
    /// let ua = HeaderValue::from_str(FIREFOX_UA).expect("Invalid UA");
    /// let auth = HeaderValue::from_str("Basic aGk6MTIzNDU2Cg==").expect("Invalid auth");
    ///
    /// let builder = DownloaderBuilder::new()
    ///     .header(header::USER_AGENT, ua)
    ///     .header(header::AUTHORIZATION, auth)
    ///     .build();
    /// ```
    ///
    /// If you need to pass in a `HeaderMap`, instead of calling `.header()` multiple times.
    /// See also [`headers()`].
    ///
    /// [`headers()`]: DownloaderBuilder::headers
    pub fn header<K: IntoHeaderName>(mut self, name: K, value: HeaderValue) -> Self {
        let mut new = self.new_header();

        new.insert(name, value);

        self.0.headers = Some(new);
        self
    }

    /// Create the [`Downloader`] with the specified options.
    pub fn build(self) -> Downloader {
        Downloader {
            directory: self.0.directory,
            retries: self.0.retries,
            concurrent_downloads: self.0.concurrent_downloads,
            style_options: self.0.style_options,
            resumable: self.0.resumable,
            headers: self.0.headers,
        }
    }
}

impl Default for DownloaderBuilder {
    fn default() -> Self {
        Self(Downloader {
            directory: std::env::current_dir().unwrap_or_default(),
            retries: Downloader::DEFAULT_RETRIES,
            concurrent_downloads: Downloader::DEFAULT_CONCURRENT_DOWNLOADS,
            style_options: StyleOptions::default(),
            resumable: true,
            headers: None,
        })
    }
}

/// Define the [`Downloader`] options.
///
/// By default, the main progress bar will stay on the screen upon completion,
/// but the child ones will be cleared once complete.
#[derive(Debug, Clone)]
pub struct StyleOptions {
    /// Style options for the main progress bar.
    main: ProgressBarOpts,
    /// Style options for the child progress bar(s).
    child: ProgressBarOpts,
}

impl Default for StyleOptions {
    fn default() -> Self {
        Self {
            main: ProgressBarOpts {
                template: Some(ProgressBarOpts::TEMPLATE_BAR_WITH_POSITION.into()),
                progress_chars: Some(ProgressBarOpts::CHARS_FINE.into()),
                enabled: true,
                clear: false,
            },
            child: ProgressBarOpts::with_pip_style(),
        }
    }
}

impl StyleOptions {
    /// Create new [`Downloader`] [`StyleOptions`].
    pub fn new(main: ProgressBarOpts, child: ProgressBarOpts) -> Self {
        Self { main, child }
    }

    /// Set the options for the main progress bar.
    pub fn set_main(&mut self, main: ProgressBarOpts) {
        self.main = main;
    }

    /// Set the options for the child progress bar.
    pub fn set_child(&mut self, child: ProgressBarOpts) {
        self.child = child;
    }

    /// Return `false` if neither the main nor the child bar is enabled.
    pub fn is_enabled(self) -> bool {
        self.main.enabled || self.child.enabled
    }
}

/// Define the options for a progress bar.
#[derive(Debug, Clone)]
pub struct ProgressBarOpts {
    /// Progress bar template string.
    template: Option<String>,
    /// Progression characters set.
    ///
    /// There must be at least 3 characters for the following states:
    /// "filled", "current", and "to do".
    progress_chars: Option<String>,
    /// Enable or disable the progress bar.
    enabled: bool,
    /// Clear the progress bar once completed.
    clear: bool,
}

impl Default for ProgressBarOpts {
    fn default() -> Self {
        Self {
            template: None,
            progress_chars: None,
            enabled: true,
            clear: true,
        }
    }
}

impl ProgressBarOpts {
    /// Template representing the bar and its position.
    ///
    ///`███████████████████████████████████████ 11/12 (99%) eta 00:00:02`
    pub const TEMPLATE_BAR_WITH_POSITION: &'static str =
        "{bar:40.blue} {pos:>}/{len} ({percent}%) eta {eta_precise:.blue}";
    /// Template which looks like the Python package installer pip.
    ///
    /// `━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ 211.23 KiB/211.23 KiB 1008.31 KiB/s eta 0s`
    pub const TEMPLATE_PIP: &'static str =
        "{bar:40.green/black} {bytes:>11.green}/{total_bytes:<11.green} {bytes_per_sec:>13.red} eta {eta:.blue}";
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

    /// Create a new [`ProgressBarOpts`].
    pub fn new(
        template: Option<String>,
        progress_chars: Option<String>,
        enabled: bool,
        clear: bool,
    ) -> Self {
        Self {
            template,
            progress_chars,
            enabled,
            clear,
        }
    }

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
        // Return a hidden Progress bar if we disabled it.
        if !self.enabled {
            return ProgressBar::hidden();
        }

        // Otherwise returns a ProgressBar with the style.
        let style = self.to_progress_style();
        ProgressBar::new(len).with_style(style)
    }

    /// Create a new [`ProgressBarOpts`] which looks like Python pip.
    pub fn with_pip_style() -> Self {
        Self {
            template: Some(ProgressBarOpts::TEMPLATE_PIP.into()),
            progress_chars: Some(ProgressBarOpts::CHARS_LINE.into()),
            enabled: true,
            clear: true,
        }
    }

    /// Set to `true` to clear the progress bar upon completion.
    pub fn set_clear(&mut self, clear: bool) {
        self.clear = clear;
    }

    /// Create a new [`ProgressBarOpts`] which hides the progress bars.
    pub fn hidden() -> Self {
        Self {
            enabled: false,
            ..ProgressBarOpts::default()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_builder_defaults() {
        let d = DownloaderBuilder::new().build();
        assert_eq!(d.retries, Downloader::DEFAULT_RETRIES);
        assert_eq!(
            d.concurrent_downloads,
            Downloader::DEFAULT_CONCURRENT_DOWNLOADS
        );
    }
}
