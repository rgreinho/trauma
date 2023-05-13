//! Changes the style of the downloader.
//!
//! Run with:
//!
//! ```not_rust
//! cargo run -q --example with-style
//! ```

use console::style;
use std::path::PathBuf;
use trauma::{
    download::Download,
    downloader::{DownloaderBuilder, ProgressBarOpts, StyleOptions},
    Error,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let debian_net_install =
        "https://cdimage.debian.org/debian-cd/current/arm64/iso-cd/debian-11.7.0-arm64-netinst.iso";
    let downloads = vec![Download::try_from(debian_net_install).unwrap()];
    let style_opts = StyleOptions::new(
        // The main bar uses a predifined template and progression characters set.
        ProgressBarOpts::new(
            Some(ProgressBarOpts::TEMPLATE_BAR_WITH_POSITION.into()),
            Some(ProgressBarOpts::CHARS_FINE.into()),
            true,
            false,
        ),
        // The child bar defines a custom template and a custom progression
        // character set using unicode block characters.
        // Other examples or symbols can easily be found online, for instance at:
        // - https://changaco.oy.lc/unicode-progress-bars/
        // - https://emojistock.com/circle-symbols/
        ProgressBarOpts::new(
            Some(format!(
                "{{bar:40.cyan/blue}} {{percent:>2.magenta}}{} ● {{eta_precise:.blue}}",
                style("%").magenta(),
            )),
            Some("●◕◑◔○".into()),
            true,
            false,
        ),
    );

    // Predefined styles can also be used.
    // let mut style_opts = StyleOptions::default();
    // style_opts.set_child(ProgressBarOpts::with_pip_style());

    let downloader = DownloaderBuilder::new()
        .directory(PathBuf::from("output"))
        .style_options(style_opts)
        .build();
    downloader.download(&downloads).await;
    Ok(())
}
