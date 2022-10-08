#!/bin/bash
set -x
set -euo pipefail

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
apt-get install wget
wget https://github.com/bazelbuild/bazelisk/releases/download/v1.14.0/bazelisk-linux-amd64
chmod +x bazelisk-linux-amd64
mv ./bazelisk-linux-amd64 /usr/bin/bazel

# Install TFLite Python deps
apt-get install --assume-yes python3-venv python3-pip
pip install numpy

chmod -R 777 $XDG_CACHE_HOME
