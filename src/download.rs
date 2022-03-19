//! Represents a file to be downloaded.

use crate::Error;
use http::StatusCode;
use std::convert::TryFrom;
use url::Url;

/// Represents a file to be downloaded.
#[derive(Debug, Clone)]
pub struct Download {
    /// URL of the file to download.
    pub url: Url,
    /// Filename used to save the file on disk.
    pub filename: String,
}

impl Download {
    /// Creates a new [`Download`].
    ///
    /// When using the [`Download::try_from`] method, the filename is
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
    /// use url::Url;
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
}

impl TryFrom<&Url> for Download {
    type Error = crate::Error;

    fn try_from(value: &Url) -> Result<Self, Self::Error> {
        value
            .path_segments()
            .ok_or_else(|| {
                Error::InvalidUrl(format!(
                    "the url \"{}\" does not contain a valid path",
                    value
                ))
            })?
            .last()
            .map(String::from)
            .map(|filename| Download {
                url: value.clone(),
                filename,
            })
            .ok_or_else(|| {
                Error::InvalidUrl(format!("the url \"{}\" does not contain a filename", value))
            })
    }
}

impl TryFrom<&str> for Download {
    type Error = crate::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Url::parse(value)
            .map_err(|e| {
                Error::InvalidUrl(format!("the url \"{}\" cannot be parsed: {}", value, e))
            })
            .and_then(|u| Download::try_from(&u))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Fail(String),
    Success,
    NotStarted,
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
}

impl Summary {
    /// Create a new [`Download`] [`Summary`].
    pub fn new(download: Download, statuscode: StatusCode, size: u64) -> Self {
        Self {
            download,
            statuscode,
            size,
            status: Status::NotStarted,
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
            status: Status::Fail(format!("{}", msg)),
            ..self
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const DOMAIN: &'static str = "http://domain.com/file.zip";

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