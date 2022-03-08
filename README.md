# Trauma

Tokio Rust Asynchronous Universal download MAnager

## Similar projects

Before starting this project, I spent some time searching the internet, trying
not to reinvent the wheel. And I did find a bunch of interesting exisiting
projects!

However they are almost all abandonned:

- DLM: <https://github.com/agourlay/dlm>
  - Active, but is just a binary/CLI tool
- Snatch: <https://github.com/derniercri/snatch>
  - Inactive since Sept '17
  - Recommend switching to [Zou]
- Zou: <https://github.com/k0pernicus/zou>
  - Inactive since Oct '17
- Duma: <https://github.com/mattgathu/duma>
  - Inactive since Nov '20
- Siwi: <https://github.com/rs-videos/siwi-download>
  - Inactive since Mar '21
- Downloader: <https://github.com/hunger/downloader>
  - Dying project
  - No answers to issues/PRs
  - Only automated updates are being merged
  - No release since Feb '21

## MVP

Here is the minimal set of features to be implemented for the project to reach
v1:

- Library only
- HTTP(S) downloads
- Download files via providing a list of URLs
  - Ability to rename downloaded files
- Ability to configure the download manager
  - Download directory
  - Maximum simultaneous requests
  - Number of retries
- Asynchronous w/ [Tokio]
- Progress bar w/ [indicatif]
  - Display the individual progress
  - Display the total progress

### Potential ideas for future versions

- Resume download
- Optional progress bar
- Support for other download protocol (i.e.: FTP)

[indicatif]: https://github.com/console-rs/indicatif
[tokio]: https://tokio.rs/
[zou]: https://github.com/k0pernicus/zou
