#!/bin/bash
set -euo pipefail

# Increase buffer size so that Git doesn't barf when pulling the tflite-support
# repo.
git config --system http.postBuffer 524288000
git config --system https.postBuffer 524288000

# Install libusb for Coral Edge TPU support
# This assumes that dpkg has been configured with an arm64 arch
apt-get install --assume-yes libusb-1.0-0-dev:$CROSS_DEB_ARCH

# Setup user
USERNAME=shyndman
USER_UID=1000
USER_GID=$USER_UID
groupadd --gid $USER_GID $USERNAME \
    && useradd --uid $USER_UID --gid $USER_GID -m $USERNAME \
    && usermod --append --groups root $USERNAME

# Create cache
mkdir -p $XDG_CACHE_HOME

# Install Bazel via Bazelisk
apt-get install --assume-yes wget
wget https://github.com/bazelbuild/bazelisk/releases/download/v1.14.0/bazelisk-linux-amd64
chmod +x bazelisk-linux-amd64
mv ./bazelisk-linux-amd64 /usr/bin/bazel

# Install TFLite Python deps
apt-get install --assume-yes python3-venv python3-pip
pip install numpy

# Ensure the cache directory is writable, as Bazelisk will create Bazel
# instances there.
chmod -R 777 $XDG_CACHE_HOME
