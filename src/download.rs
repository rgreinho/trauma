//! Represents a file to be downloaded.

use crate::{Error, ResponseExt};
use bon::Builder;
use reqwest::{IntoUrl, StatusCode, Url};
use reqwest_middleware::ClientWithMiddleware;

/// Represents a file to be downloaded.
#[derive(Debug, Clone, Builder)]
#[builder(on(String, into))]
pub struct Download {
    /// URL of the file to download.
    #[builder(with = |value: impl IntoUrl| -> Result<_, Error> {
        value.into_url().map_err(|e| Error::InvalidUrl(format!("the url cannot be parsed: {e}")))
    })]
    url: Url,
    /// File name used to save the file on disk. Overrides the file name
    /// extracted from the URL.
    filename_override: Option<String>,
}

impl Download {
    pub async fn head(
        &self,
        client: &ClientWithMiddleware,
    ) -> Result<reqwest::Response, reqwest_middleware::Error> {
        client.head(self.url.clone()).send().await
    }

    pub async fn get(
        &self,
        client: &ClientWithMiddleware,
    ) -> Result<reqwest::Response, reqwest_middleware::Error> {
        client.get(self.url.clone()).send().await
    }

    /// Check whether the download is resumable.
    pub fn is_resumable(response: &reqwest::Response) -> bool {
        let accept_ranges = response.accept_ranges();

        // If the server doesn't support range requests, we consider the
        // download as not resumable.
        if !accept_ranges {
            return false;
        }

        // If we don't get a content length or if we get a content length of 0,
        // we also consider the download as not resumable.
        let content_length = response.content_length_header();
        content_length.is_some() && content_length != Some(0)
    }

    /// Get the filename from the `Content-Disposition` header.
    ///
    /// Returns `None` if the header is not set or if its value cannot be parsed.
    pub fn filename_from_content_disposition(response: &reqwest::Response) -> Option<String> {
        if let Some(value) = response.content_disposition() {
            return Download::parse_content_disposition(&value);
        }

        None
    }

    /// Parse the value of the `Content-Dispostion` header.
    ///
    /// Attempts to extract the filename form the header value.
    fn parse_content_disposition(value: &str) -> Option<String> {
        if let Some(filename) = value.split(';').find_map(|part| {
            if part.trim().starts_with("filename=") {
                return Some(
                    part.trim()
                        .split('=')
                        .nth(1)
                        .unwrap()
                        .trim_matches('"')
                        .to_string(),
                );
            }
            None
        }) {
            return Some(filename);
        }

        None
    }

    /// Get the filename from the URL.
    ///
    /// Returns an error if the URL does not contain a filename.
    pub fn filename_from_url(&self) -> Result<String, Error> {
        let path = self.url.path();

        // Check for root path early.
        if path == "/" || path.is_empty() {
            return Err(Error::InvalidUrl(format!(
                "the URL \"{}\" has no filename",
                self.url
            )));
        }

        // Get the last segment item.
        self.url
            .path_segments()
            .ok_or_else(|| Error::InvalidUrl(format!("not an absolute URL: {}", self.url)))?
            .next_back()
            .map(String::from)
            .ok_or_else(|| Error::InvalidUrl(format!("the URL \"{}\" has no filename", self.url)))
    }

    /// Get the filename from multiple sources.
    ///
    /// Here is the precedence order:
    ///   - filename override
    ///   - filename from `Content-Disposition` header
    ///   - filename from URL
    ///
    /// Returns `None` if the name cannot be determined.
    pub fn infer_filename(&self, response: &reqwest::Response) -> Option<String> {
        if self.filename_override.is_some() {
            return self.filename_override.clone();
        }

        let filename = Download::filename_from_content_disposition(response);
        if filename.is_some() {
            return filename;
        }

        self.filename_from_url().ok()
    }

    /// Get the filename override if any.
    pub fn filename_override(&self) -> Option<&String> {
        self.filename_override.as_ref()
    }

    /// Get the URL.
    ///
    /// Returns a clone of the URL.
    pub fn url(&self) -> Url {
        self.url.clone()
    }

    /// Get the URL as &str.
    pub fn url_as_str(&self) -> &str {
        self.url.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Status {
    Fail(String),
    #[default]
    NotStarted,
    Skipped(String),
    Success,
}
/// Represents a [`Download`] summary.
#[derive(Debug, Clone, Builder)]
pub struct Summary {
    /// Downloaded items.
    download: Download,
    /// HTTP status code.
    #[builder(default = StatusCode::PROCESSING)]
    statuscode: StatusCode,
    /// Download size in bytes.
    #[builder(default)]
    size: u64,
    /// Status.
    #[builder(default)]
    status: Status,
    /// Resumable.
    #[builder(default)]
    resumable: bool,
}

impl Summary {
    /// Attach a status to a [`Download`] [`Summary`].
    pub fn with_status(self, status: Status) -> Self {
        Self { status, ..self }
    }

    /// Get the summary's status.
    pub fn statuscode(&self) -> StatusCode {
        self.statuscode
    }

    /// Get the summary's size.
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Get a reference to the summary's download.
    pub fn download(&self) -> &Download {
        &self.download
    }

    /// Get a reference to the summary's status.
    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn fail(self, msg: impl std::fmt::Display) -> Self {
        Self::builder()
            .download(self.download)
            .status(Status::Fail(format!("{msg}")))
            .build()
    }

    /// Set the summary's resumable.
    pub fn set_resumable(&mut self, resumable: bool) {
        self.resumable = resumable;
    }

    /// Get the summary's resumable.
    #[must_use]
    pub fn resumable(&self) -> bool {
        self.resumable
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const DOMAIN: &str = "http://domain.com/file.zip";

    #[test]
    fn test_builder_from_url_as_str() {
        let d = Download::builder().url(DOMAIN).unwrap().build();
        assert_eq!(d.filename_from_url().unwrap(), "file.zip".to_string())
    }

    #[test]
    fn test_parse_content_disposition() {
        let value = "attachment; filename=VSCodeUserSetup-x64-1.124.2.exe; filename*=UTF-8''VSCodeUserSetup-x64-1.124.2.exe";
        let filename = Download::parse_content_disposition(value);
        assert_eq!(
            filename,
            Some("VSCodeUserSetup-x64-1.124.2.exe".to_string())
        )
    }
}
