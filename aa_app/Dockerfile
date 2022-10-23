ARG PLATFORM_BASE_IMAGE
FROM ${PLATFORM_BASE_IMAGE}

ARG LIBSSL_PKG_NAME
RUN apt-get update
RUN apt-get install --assume-yes --no-install-recommends \
        apt-utils apt-transport-https gnupg \
        libgstreamer1.0-0 gstreamer1.0-libav gstreamer1.0-plugins-base \
        gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly \
        libgstrtspserver-1.0-0 libges-1.0-0 ${LIBSSL_PKG_NAME} libusb-1.0-0

ARG BUILD_TARGET_PROFILE_PATH
RUN mkdir --parents /app/lib
COPY ${BUILD_TARGET_PROFILE_PATH}/arena_autocam /app/arena_autocam
# COPY ${BUILD_TARGET_PROFILE_PATH}/lib /app/lib
CMD /app/arena_autocam
