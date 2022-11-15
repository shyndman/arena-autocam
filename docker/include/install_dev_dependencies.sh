#!/bin bash

set -x
set -e

if [[ ! -v TARGETARCH ]]; then
  echo "TARGETARCH not set"
  exit 1
fi

apt-get update && \
  apt-get --assume-yes install \
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
