FROM balenalib/raspberrypi4-64-debian:bullseye-run as run
RUN install_packages apt-utils libssl-dev apt-transport-https gnupg \
    libgstreamer1.0-0 gstreamer1.0-libav \
    gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly \
    libgstrtspserver-1.0-dev libges-1.0-dev libssl-dev libusb-1.0-0

RUN mkdir -fp /app/lib
COPY target/aarch64-unknown-linux-gnu/release/arena_autocam /app/arena_autocam
COPY target/aarch64-unknown-linux-gnu/release/lib /app/lib

CMD /app/arena_autocam
