apt update &&
    apt install -y gcc-mingw-w64 pkg-config &&
    rustup target add x86_64-pc-windows-gnu &&
    cargo build -r --target x86_64-pc-windows-gnu &&
    mv /mnt/target/x86_64-pc-windows-gnu/release/coomer-download.exe /mnt/coomer-win-amd64.exe