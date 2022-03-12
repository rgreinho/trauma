//! Simple download example.
//!
//! Run with
//!
//! ```not_rust
//! cargo run --example simple
//! ```
//!
use color_eyre::{eyre::Report, Result};
use std::path::PathBuf;
use tracing_subscriber;
use trauma::{download::Download, downloader::DownloaderBuilder};
// use opentelemetry::sdk::export::trace::stdout;
// use tracing_subscriber::layer::SubscriberExt;
// use tracing_subscriber::Registry;

#[tokio::main]
async fn main() -> Result<(), Report> {
    // Setup the application.
    color_eyre::install()?;

    // Prepare the logger.
    // Currently commented out due to a bug with the reqwest-tracing crate:
    // https://github.com/TrueLayer/reqwest-middleware/issues/35
    // let tracer = stdout::new_pipeline().install_simple();
    // let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    // let subscriber = Registry::default().with(telemetry);
    // tracing::subscriber::set_global_default(subscriber).unwrap();
    //
    // Falling back to the regular tracing library.
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Download the file(s).
    let reqwest_rs = "https://github.com/seanmonstar/reqwest/archive/refs/tags/v0.11.9.zip";
    let downloads = vec![Download::try_from(reqwest_rs).unwrap()];
    let downloader = DownloaderBuilder::new()
        .directory(PathBuf::from("output"))
        .build();
    downloader.download(&downloads).await;

    Ok(())
}
