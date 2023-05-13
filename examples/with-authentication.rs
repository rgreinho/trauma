//! Download a file with authentication example.
//!
//! Setup for this example:
//!
//! From the root of the project:
//! ```not_rust
//! mkdir -p examples/miniserve
//! cd examples/miniserve
//! curl -sLO https://cdimage.debian.org/debian-cd/current/arm64/iso-cd/debian-11.7.0-arm64-netinst.iso
//! miniserve --auth trauma:test .
//! ```
//!
//! Then from another terminal:
//!
//! ```not_rust
//! cargo run -q --example simple
//! ```
//!
//! The value for the authentication header can be generated from the command line:
//! ```not_rust
//! echo -n "trauma:test"|base64
//! ```
//! Or from a website like https://www.debugbear.com/basic-auth-header-generator.
//!
//! Miniserve is a utility written in rust to serve files over HTTP:
//! https://github.com/svenstaro/miniserve
//!

use reqwest::header::{self, HeaderValue};
use std::path::PathBuf;
use trauma::{download::Download, downloader::DownloaderBuilder, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let reqwest_rs = "http://localhost:8080/debian-11.7.0-arm64-netinst.iso";
    let downloads = vec![Download::try_from(reqwest_rs).unwrap()];
    let auth = HeaderValue::from_str("Basic dHJhdW1hOnRlc3Q=").expect("Invalid auth");
    let downloader = DownloaderBuilder::new()
        .directory(PathBuf::from("output"))
        .header(header::AUTHORIZATION, auth)
        .build();
    let summaries = downloader.download(&downloads).await;
    let summary = summaries.first().unwrap();
    println!("{:?}", summary.status());
    Ok(())
}
