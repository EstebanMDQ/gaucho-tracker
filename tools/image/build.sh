#!/bin/bash
set -e

# Path to pi-gen
PIGEN_DIR="./pi-gen"

# Ensure pi-gen is cloned
if [ ! -d "$PIGEN_DIR" ]; then
  echo "Cloning pi-gen..."
  git clone https://github.com/RPi-Distro/pi-gen.git "$PIGEN_DIR"
fi

# Build the Rust application
echo "Building Gaucho Tracker Rust application..."
cargo build --release --manifest-path ./Cargo.toml

# Copy the Rust binary to the custom stage
echo "Copying Gaucho Tracker binary..."
mkdir -p ./tools/image/stage-gaucho/01-gaucho-tracker/files/usr/local/bin
cp ./target/release/gaucho-tracker ./tools/image/stage-gaucho/01-gaucho-tracker/files/usr/local/bin/

# Ensure the required directory structure exists
echo "Ensuring directory structure exists..."
mkdir -p "$PIGEN_DIR/work/gaucho-os-lite/stage0/rootfs/etc/apt/"

# Copy config into pi-gen
cp ./tools/image/config "$PIGEN_DIR/config"

# Copy custom stage
cp -r ./tools/image/stage-gaucho "$PIGEN_DIR/"

# Detect the operating system
OS=$(uname -s)

# Use sudo only if running on Linux
if [ "$OS" = "Linux" ]; then
  SUDO="sudo"
else
  SUDO=""
fi

# Build image using Docker
cd "$PIGEN_DIR"
$SUDO ./build-docker.sh
