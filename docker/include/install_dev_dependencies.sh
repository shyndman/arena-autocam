#!/bin/bash

set -x
set -e

if [[ ! -v TARGETARCH ]]; then
  echo "TARGETARCH not set"
  exit 1
fi

apt-get install \
  libcairo2-dev:$TARGETARCH \
  libgstreamer1.0-dev:$TARGETARCH \
  gstreamer1.0-plugins-base:$TARGETARCH \
  gstreamer1.0-plugins-good:$TARGETARCH \
  gstreamer1.0-plugins-bad:$TARGETARCH \
  gstreamer1.0-plugins-ugly:$TARGETARCH \
  gstreamer1.0-libav:$TARGETARCH \
  libgstrtspserver-1.0-dev:$TARGETARCH \
  libges-1.0-dev:$TARGETARCH \
  libssl-dev:$TARGETARCH \
  libusb-1.0-0-dev:$TARGETARCH # Install libusb for Coral Edge TPU support

# Increase buffer size so that Git doesn't barf when pulling the tflite-support
# repo.
git config --system http.postBuffer 524288000
git config --system https.postBuffer 524288000

# Tflite build requirements
# Bazelisk
wget https://github.com/bazelbuild/bazelisk/releases/download/v1.14.0/bazelisk-linux-amd64
chmod +x bazelisk-linux-amd64
mv ./bazelisk-linux-amd64 /usr/bin/bazel

# Python deps
apt-get install python3-pip
pip3 install numpy
