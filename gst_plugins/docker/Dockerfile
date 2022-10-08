FROM ghcr.io/cross-rs/aarch64-unknown-linux-gnu:main

COPY docker/install_deb.sh /
# Change the packages to your dependencies.
RUN /install_deb.sh arm64  libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
      libgstreamer1.0-0 gir1.2-gstreamer-1.0 \
      gstreamer1.0-plugins-base gstreamer1.0-plugins-good \
      gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly \
      gstreamer1.0-libav libgstrtspserver-1.0-dev libges-1.0-dev
