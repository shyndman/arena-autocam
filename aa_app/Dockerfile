ARG PLATFORM_BASE_IMAGE
FROM ${PLATFORM_BASE_IMAGE}

# This is a super hack to get builds going. Apparently buildx is
# misunderstanding how PATH works
RUN ln -s /usr/bin/dpkg-split /usr/sbin/dpkg-split
RUN ln -s /usr/bin/dpkg-deb /usr/sbin/dpkg-deb
RUN ln -s /bin/tar /usr/sbin/tar
RUN ln -s /bin/rm /usr/sbin/rm

# Install all the packages we need
ARG LIBSSL_PKG_NAME
RUN install_packages dialog apt-utils
RUN install_packages apt-transport-https gnupg \
        libgstreamer1.0-0 gstreamer1.0-libav gstreamer1.0-plugins-base \
        gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly \
        libgstrtspserver-1.0-0 libges-1.0-0 \
        libcamera-dev v4l-utils \
        ${LIBSSL_PKG_NAME} libusb-1.0-0

# Copy over necessary files from the build
ARG BUILD_TARGET_PROFILE_PATH
RUN mkdir --parents /app/lib
COPY ${BUILD_TARGET_PROFILE_PATH}/arena_autocam /app/arena_autocam
# COPY ${BUILD_TARGET_PROFILE_PATH}/lib /app/lib

# Add a little start script that manages udev
RUN bash -c '\
    cat << EOF > /app/start \
    #!/bin/sh \
    udevadm control --reload \
    /app/arena_autocam \
    EOF; \
    chmod +x /app/start'

# This, along with privileged=true in the docker-compose.yaml, ensures that the
# container can access devices (like the camera)
ENV UDEV=on

CMD /app/start
