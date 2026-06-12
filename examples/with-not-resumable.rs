//! Download example where a file cannot be resumed.
//!
//! Run with
//!
//! ```not_rust
//! cargo run -q --example with-not-resumable
//! ```

use color_eyre::Result;
use comfy_table::{Row, Table};
use std::path::PathBuf;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use tracing::debug;
use trauma::{
    download::{Download, Status, Summary},
    downloader::Downloader,
    Error,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Setup logging.
    tracing_subscriber::fmt::fmt()
        .with_env_filter("with_not_resumable=debug,trauma=debug")
        .init();

    // Prepare the download.
    // This URL is known to return a Content-Length: 0 to HEAD requests.
    let url = "https://stateparks.oregon.gov/index.cfm?do=main.loadFile&load=_siteFiles%2Fpublications%2F%2FSpringValleyGeoPDF093621.pdf";
    let output_dir = PathBuf::from("output");
    let filename = "SpringValleyGeoPDF093621.pdf";
    let output = output_dir.join(filename);

    // Clean up.
    let _ = fs::remove_file(&output).await;
    fs::create_dir_all(&output_dir).await?;

    // Create a corrupted file for testing that it does get re-downloaded.
    let mut file = File::create(&output).await?;
    file.write_all(b"hello").await?;
    file.flush().await?;
    debug!(
        "Corrupted file created: {} bytes.",
        file.metadata().await?.len()
    );

    // Setup the download item for Trauma.
    let downloads = vec![Download::builder()
        .url(url)?
        .filename_override(filename)
        .build()];
    let downloader = Downloader::builder().directory(output_dir).build();

    // First attempt, redownload the file and display the summary.
    print!("Downloading file... ");
    let summaries = downloader.download(&downloads).await;
    display_summary(&summaries);

    // Second attempt, the file is already fully downloaded, so it should be skipped.
    println!("Downloading file again... ");
    let summaries = downloader.download(&downloads).await;
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
