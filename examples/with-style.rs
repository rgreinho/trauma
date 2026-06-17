//! Changes the style of the downloader.
//!
//! Run with:
//!
//! ```not_rust
//! cargo run -q --example with-style
//! ```

use console::style;
use trauma::{
    download::Download,
    downloader::{Downloader, ProgressBarOpts, StyleOptions},
    Error,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let debian_net_install =
        "https://cdimage.debian.org/debian-cd/current/amd64/iso-cd/debian-13.5.0-amd64-netinst.iso";
    let downloads = vec![Download::builder().url(debian_net_install)?.build()];
    let style_opts = StyleOptions::builder()
        // The main bar uses a predefined template and progression characters set.
        .main(
            ProgressBarOpts::builder()
                .template(ProgressBarOpts::TEMPLATE_BAR_WITH_POSITION)
                .progress_chars(ProgressBarOpts::CHARS_FINE)
                .build(),
        )
        // The child bar defines a custom template and a custom progression
        // character set using unicode block characters.
        // Other examples or symbols can easily be found online, for instance at:
        // - https://changaco.oy.lc/unicode-progress-bars/
        // - https://emojistock.com/circle-symbols/
        .child(
            ProgressBarOpts::builder()
                .template(format!(
                    "{{bar:40.cyan/blue}} {{percent:>2.magenta}}{} ● {{eta_precise:.blue}} {{msg}}",
                    style("%").magenta(),
                ))
                .progress_chars("●◕◑◔○")
                .clear()
                .build(),
        )
        .build();

    // Predefined styles can also be used.
    // let mut style_opts = StyleOptions::default();
    // style_opts.set_child(ProgressBarOpts::with_pip_style());

    let downloader = Downloader::builder()
        .directory("output")
        .style_options(style_opts)
        .build();
    downloader.download(&downloads).await;

    Ok(())
}
