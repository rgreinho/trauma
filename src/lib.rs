//! Trauma is crate aiming at providing a simple way to download files asynchronously via HTTP(S).
//!

pub mod download;
pub mod downloader;

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
