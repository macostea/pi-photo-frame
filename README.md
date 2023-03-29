# Pi Photo Frame
[![Build status](https://github.com/macostea/pi-photo-frame/actions/workflows/build-app.yml/badge.svg?branch=master)](https://github.com/macostea/pi-photo-frame/actions/workflows/build-app.yml)

A Raspberry Pi powered smart photo frame that displays random photos with extra metadata, built with Rust and GTK4.

# Installation

GTK4 is not available in the bullseye-based Raspberry Pi OS, so you have a few options:
1. Install the bullseye-based Raspberry Pi, update your sources to bookworm and upgrade the system (tested)
2. Flash a [bookworm image](http://raspi.debian.net/tested-images/)
3. Compile GTK4 for your system

After you install a system with GTK4 you only need to:
1. [Download a release](https://github.com/macostea/pi-photo-frame/releases/latest) from this repo
2. Extract the file `tar -xzvf pi-photo-builder.tar.gz`
3. Install the deb package `dpkg -i target/aarch64-unknown-linux-gnu/debian/*.deb`

# Configuration

The application can be configured by editing the configuration file in `/etc/pi-photo-frame.json5`. You can find an example in the [config.json5](config.json5) file in this repo.

# Start the application

Pi Photo Frame is installed as a systemd unit so you can use systemd to manage it.

### Open at startup
```bash
systemctl enable photo-frame
```

### Start the application
```bash
systemctl start photo-frame
```

