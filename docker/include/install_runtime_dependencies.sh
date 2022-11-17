#!/bin/bash

set -x
set -e

apt-get update && \
    apt-get --assume-yes --no-install-recommends install \
    gstreamer1.0-libav \
    gstreamer1.0-plugins-bad \
    gstreamer1.0-plugins-base \
    gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-ugly \
    libcairo2 \
    libges-1.0-0 \
    libgstrtspserver-1.0-0 \
    libusb-1.0-0 `# Install libusb for Coral Edge TPU support` \
    ;

# Clean up apt metadata
rm -rf /var/cache/apt/archives/* /var/lib/apt/lists/*
