#!/bin/sh

cargo build --release

unset RUSTC_WRAPPER
cargo deb -v
