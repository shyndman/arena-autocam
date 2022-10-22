FROM ghcr.io/cross-rs/aarch64-unknown-linux-gnu:main

COPY docker/install_tflite_build_deps.sh /
# Change the packages to your dependencies.
RUN apt-get update && apt-get install -y --no-install-recommends \
    apt-utils libssl-dev apt-transport-https gnupg \
    # For xxd
    vim
ENV XDG_CACHE_HOME /home/cross
RUN /install_tflite_build_deps.sh
RUN dpkg --add-architecture arm64
RUN apt-get update
RUN DEBIAN_FRONTEND=noninteractive apt-get -y install \
    libgstreamer1.0-dev:arm64 \
    gstreamer1.0-plugins-base:arm64 gstreamer1.0-plugins-good:arm64 \
    gstreamer1.0-plugins-bad:arm64 gstreamer1.0-plugins-ugly:arm64 \
    gstreamer1.0-libav:arm64 libgstrtspserver-1.0-dev:arm64 libges-1.0-dev:arm64 \
    libssl-dev:arm64 libusb-1.0-0-dev:arm64
