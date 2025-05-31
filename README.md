
## Cooomer Downloader

A command line tool for downloading content from coomer platforms. This project is a personal tool I developed a few months ago and am now sharing for others to use.

### Platform Support

- **Windows**: Pre-compiled binaries are available.
- **macOS**: Not supported (no testing environment available).
- **Linux**: Requires manual compilation using Rust.

### Compilation Instructions

#### Debian/Ubuntu (Linux)
Install dependencies and build the project using the following commands:

```bash
apt update && \
apt install -y cargo git gcc pkg-config libssl-dev && \
git clone https://github.com/baconator696/coomer-download-rust.git && \
cd coomer-download-rust && \
cargo build -r
```

The compiled binary will be located at `./coomer-download-rust/target/release/coomer-download`.

#### Other Operating Systems
If you're using a different OS, ensure Rust is installed. The compilation process is similar to the Debian/Ubuntu instructions above.

### Notes
I rarely use this tool, so support is minimal. If you encounter issues, you may need to debug and resolve them yourself.
