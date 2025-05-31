# linux build
# apt install rustup build-essential libssl-dev libzstd-dev zlib1g-dev pkg-config
# rustup target add aarch64-unknown-linux-gnu x86_64-unknown-linux-gnu x86_64-pc-windows-gnullvm aarch64-pc-windows-gnullvm
#
# arm
#   dpkg --add-architecture arm64 && apt update
#   apt install gcc-aarch64-linux-gnu libssl-dev:arm64 -y
#   cp -rs /usr/lib/aarch64-linux-gnu/* /usr/aarch64-linux-gnu/lib
OPENSSL_DIR=/usr/aarch64-linux-gnu cargo build -r --target aarch64-unknown-linux-gnu
# amd 
cargo build -r --target x86_64-unknown-linux-gnu
# windows build
# amd
cargo build -r --target x86_64-pc-windows-gnullvm
# arm
cargo build -r --target aarch64-pc-windows-gnullvm