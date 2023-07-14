#!/bin/sh

cargo build --features=sentry-native --profile=release-with-debug --target=aarch64-unknown-linux-gnu

export LD_LIBRARY_PATH=/app/rootfs/usr/lib/aarch64-linux-gnu
cargo deb --no-build --profile=release-with-debug --separate-debug-symbols --target=aarch64-unknown-linux-gnu -v
