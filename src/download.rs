//! Represents a file to be downloaded.

use crate::Error;
use reqwest::{
    header::{ACCEPT_RANGES, CONTENT_LENGTH},
    StatusCode, Url,
};
use reqwest_middleware::ClientWithMiddleware;
use std::convert::TryFrom;

/// Represents a file to be downloaded.
#[derive(Debug, Clone)]
pub struct Download {
    /// URL of the file to download.
    pub url: Url,
    /// File name used to save the file on disk.
    pub filename: String,
}

impl Download {
    /// Creates a new [`Download`].
    ///
    /// When using the [`Download::try_from`] method, the file name is
    /// automatically extracted from the URL.
    ///
    /// ## Example
    ///
    /// The following calls are equivalent, minus some extra URL validations
    /// performed by `try_from`:
    ///
    /// ```no_run
    /// # use color_eyre::{eyre::Report, Result};
    /// use trauma::download::Download;
    /// use reqwest::Url;
    ///
    /// # fn main() -> Result<(), Report> {
    /// Download::try_from("https://example.com/file-0.1.2.zip")?;
    /// Download::new(&Url::parse("https://example.com/file-0.1.2.zip")?, "file-0.1.2.zip");
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(url: &Url, filename: &str) -> Self {
        Self {
            url: url.clone(),
            filename: String::from(filename),
        }
    }

    /// Check whether the download is resumable.
    pub async fn is_resumable(
        &self,
        client: &ClientWithMiddleware,
    ) -> Result<bool, reqwest_middleware::Error> {
        let res = client.head(self.url.clone()).send().await?;
        let headers = res.headers();
        match headers.get(ACCEPT_RANGES) {
            None => Ok(false),
            Some(x) if x == "none" => Ok(false),
            Some(_) => Ok(true),
        }
    }

    /// Retrieve the content_length of the download.
    ///
    /// Returns None if the "content-length" header is missing or if its value
    /// is not a u64.
    pub async fn content_length(
        &self,
        client: &ClientWithMiddleware,
    ) -> Result<Option<u64>, reqwest_middleware::Error> {
        let res = client.head(self.url.clone()).send().await?;
        let headers = res.headers();
        match headers.get(CONTENT_LENGTH) {
            None => Ok(None),
            Some(header_value) => match header_value.to_str() {
                Ok(v) => match v.to_string().parse::<u64>() {
                    Ok(v) => Ok(Some(v)),
                    Err(_) => Ok(None),
                },
                Err(_) => Ok(None),
            },
        }
    }
}

impl TryFrom<&Url> for Download {
    type Error = crate::Error;

    fn try_from(value: &Url) -> Result<Self, Self::Error> {
        value
            .path_segments()
            .ok_or_else(|| {
                Error::InvalidUrl(format!("the url \"{value}\" does not contain a valid path"))
            })?
            .next_back()
            .map(String::from)
            .map(|filename| Download {
                url: value.clone(),
                filename: form_urlencoded::parse(filename.as_bytes())
                    .map(|(key, val)| [key, val].concat())
                    .collect(),
            })
            .ok_or_else(|| {
                Error::InvalidUrl(format!("the url \"{value}\" does not contain a filename"))
            })
    }
}

impl TryFrom<&str> for Download {
    type Error = crate::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Url::parse(value)
            .map_err(|e| Error::InvalidUrl(format!("the url \"{value}\" cannot be parsed: {e}")))
            .and_then(|u| Download::try_from(&u))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Fail(String),
    NotStarted,
    Skipped(String),
    Success,
}
/// Represents a [`Download`] summary.
#[derive(Debug, Clone)]
pub struct Summary {
    /// Downloaded items.
    download: Download,
    /// HTTP status code.
    statuscode: StatusCode,
    /// Download size in bytes.
    size: u64,
    /// Status.
    status: Status,
    /// Resumable.
    resumable: bool,
}

impl Summary {
    /// Create a new [`Download`] [`Summary`].
    pub fn new(download: Download, statuscode: StatusCode, size: u64, resumable: bool) -> Self {
        Self {
            download,
            statuscode,
            size,
            status: Status::NotStarted,
            resumable,
        }
    }

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
        Self {
            status: Status::Fail(format!("{msg}")),
            ..self
        }
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
    fn test_try_from_url() {
        let u = Url::parse(DOMAIN).unwrap();
        let d = Download::try_from(&u).unwrap();
        assert_eq!(d.filename, "file.zip")
    }

    #[test]
    fn test_try_from_string() {
        let d = Download::try_from(DOMAIN).unwrap();
        assert_eq!(d.filename, "file.zip")
    }
}
