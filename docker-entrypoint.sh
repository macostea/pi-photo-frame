#!/bin/sh

cargo build --release
cargo deb -v
