#!/bin/sh

cargo build --release --target=aarch64-unknown-linux-gnu

export LD_LIBRARY_PATH=/app/rootfs/usr/lib/aarch64-linux-gnu
cargo deb --target=aarch64-unknown-linux-gnu -v
