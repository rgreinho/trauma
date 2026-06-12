//! Download files and show the report using comfy-table.
//!
//! Run with:
//!
//! ```not_rust
//! cargo run -q --example with-report
//! ```

use color_eyre::{eyre::Report, Result};
use comfy_table::{Row, Table};
use trauma::{
    download::{Download, Status, Summary},
    downloader::Downloader,
};

#[tokio::main]
async fn main() -> Result<(), Report> {
    // Setup the application.
    color_eyre::install()?;

    // Download the file(s).
    let reqwest_rs = "https://github.com/seanmonstar/reqwest/archive/refs/tags/v0.11.9.zip";
    let fake = format!("{}.fake", &reqwest_rs);
    let downloads = vec![
        Download::builder().url(reqwest_rs)?.build(),
        Download::builder().url(reqwest_rs)?.build(),
        Download::builder().url(fake)?.build(),
    ];
    let downloader = Downloader::builder().directory("output").build();
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
            Status::Skipped(s) => {
                error = s.to_string();
                String::from("⏭️")
            }
        };
        table.add_row(vec![
            &s.download()
                .filename_override()
                .map_or("", |v| v)
                .to_string(),
            &s.size().to_string(),
            &status,
            &error,
        ]);
    });
    println!("{table}");
}
