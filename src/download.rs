//! Represents a file to be downloaded.

use crate::Error;
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
