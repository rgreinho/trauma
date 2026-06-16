//! Trauma is crate aiming at providing a simple way to download files
//! asynchronously via HTTP(S).

pub mod download;
pub mod downloader;

use reqwest::{
    header::{ACCEPT_RANGES, CONTENT_LENGTH},
    Url,
};
use std::io;
use thiserror::Error;

/// Errors that can happen when using Trauma.
#[derive(Error, Debug)]
pub enum Error {
    /// Error from an underlying system.
    #[error("Internal error: {0}")]
    Internal(String),
    /// Error from the underlying URL parser or the expected URL format.
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    /// I/O Error.
    #[error("I/O error")]
    IOError {
        #[from]
        source: io::Error,
    },
    /// Error from the Reqwest library.
    #[error("Reqwest Error")]
    Reqwest {
        #[from]
        source: reqwest::Error,
    },
}

/// Extension trait for `reqwest::header::HeaderMap`
///
/// Provide convenient methods for accessing common headers values.
pub trait ResponseExt {
    /// Get the content length from the headers.
    ///
    /// Returns None if the "content-length" header is missing or if its value
    /// is not a u64.
    fn content_length_header(&self) -> Option<u64>;

    /// Check whether the server supports range requests.
    ///
    /// Returns false if the "accept-ranges" header is missing or if its value
    /// is "none".
    fn accept_ranges(&self) -> bool;

    /// Get the location header from the headers.
    ///
    /// Returns None if the "location" header is missing or if its value cannot
    /// be parsed as a URL.
    fn location(&self) -> Option<Url>;

    /// Get the content disposition header from the headers.
    ///
    /// Returns None if the "content-disposition" header is missing or if its
    /// value cannot be parsed as a string.
    fn content_disposition(&self) -> Option<String>;
}

impl ResponseExt for reqwest::Response {
    fn content_length_header(&self) -> Option<u64> {
        self.headers()
            .get(CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
    }

    fn accept_ranges(&self) -> bool {
        match self.headers().get(ACCEPT_RANGES) {
            None => false,
            Some(x) if x == "none" => false,
            Some(_) => true,
        }
    }

    fn location(&self) -> Option<Url> {
        self.headers()
            .get(reqwest::header::LOCATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| self.url().join(s).ok())
    }

    fn content_disposition(&self) -> Option<String> {
        self.headers()
            .get(reqwest::header::CONTENT_DISPOSITION)
            .and_then(|v| v.to_str().ok())
            .map(String::from)
    }
}
