#!/bin/bash
set -e

# Path to pi-gen
PIGEN_DIR="./pi-gen"

# Ensure pi-gen is cloned
if [ ! -d "$PIGEN_DIR" ]; then
  echo "Cloning pi-gen..."
  git clone https://github.com/RPi-Distro/pi-gen.git "$PIGEN_DIR"
fi

# Copy config into pi-gen
cp ./tools/image/config "$PIGEN_DIR/config"

# Copy custom stage
cp -r ./tools/image/stage-gaucho "$PIGEN_DIR/"

# Build image using Docker
cd "$PIGEN_DIR"
./build-docker.sh
