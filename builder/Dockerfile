FROM rust:bookworm

RUN rustup update

RUN apt-get update && apt-get install -y build-essential pkg-config
RUN cargo install cargo-deb

WORKDIR /app

RUN rustup target add aarch64-unknown-linux-gnu
RUN apt-get update && apt-get install -y g++-aarch64-linux-gnu libc6-dev-arm64-cross

ADD rootfs /app/rootfs

ENV PKG_CONFIG_SYSROOT_DIR=/app/rootfs CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc BINDGEN_EXTRA_CLANG_ARGS_aarch64_unknown_linux_gnu=--sysroot=/app/rootfs/usr/lib/aarch64-linux-gnu CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS=-Clink-args=--sysroot=/app/rootfs PKG_CONFIG_PATH=/app/rootfs/usr/lib/aarch64-linux-gnu/pkgconfig/:/app/rootfs/usr/share/pkgconfig/: CPATH=/app/rootfs/usr/include/aarch64-linux-gnu
ADD docker-entrypoint.sh /docker-entrypoint.sh

RUN chmod +x /docker-entrypoint.sh

WORKDIR /builder

ENTRYPOINT ["/docker-entrypoint.sh"]