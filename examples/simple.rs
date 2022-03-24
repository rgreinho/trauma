//! Simple download example.
//!
//! Run with
//!
//! ```not_rust
//! cargo run --example simple
//! ```
//!
use color_eyre::{eyre::Report, Result};
use comfy_table::{Row, Table};
use std::path::PathBuf;
use tracing_subscriber;
use trauma::{
    download::{Download, Status, Summary},
    downloader::DownloaderBuilder,
};
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
        .with_env_filter("trauma=debug")
        .init();

    // Download the file(s).
    let reqwest_rs = "https://github.com/seanmonstar/reqwest/archive/refs/tags/v0.11.9.zip";
    let fake = format!("{}.fake", &reqwest_rs);
    let downloads = vec![
        Download::try_from(reqwest_rs).unwrap(),
        Download::try_from(fake.as_str()).unwrap(),
    ];
    let downloader = DownloaderBuilder::new()
        .directory(PathBuf::from("output"))
        .build();
    let summaries = downloader.download(&downloads).await;

    // Display results.
    display_summary(&summaries);

    Ok(())
}

fn display_summary(summaries: &[Summary]) {
    let mut table = Table::new();
    let header = Row::from(vec!["File", "Size", "Status", "Error"]);
    table.set_header(header);
    summaries.iter().for_each(|s| {
        let mut error = String::new();
        let status = match s.status() {
            Status::Success => String::from("✅"),
            Status::Fail(s) => {
                error = s.to_string();
                error.truncate(50);
                if error.len() <= 50 {
                    error.push_str("...");
                }
                String::from("❌")
            }
            Status::NotStarted => String::from("🔜"),
        };
        table.add_row(vec![
            &s.download().filename,
            &s.size().to_string(),
            &status,
            &error,
        ]);
        ()
    });
    println!("{table}");
}
