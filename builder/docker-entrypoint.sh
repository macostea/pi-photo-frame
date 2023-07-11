#!/bin/sh

cargo build --profile=release-with-debug --target=aarch64-unknown-linux-gnu

export LD_LIBRARY_PATH=/app/rootfs/usr/lib/aarch64-linux-gnu
cargo deb --profile=release-with-debug --separate-debug-symbols --target=aarch64-unknown-linux-gnu -v
