# syntax=docker/dockerfile:1
ARG DOCKER_REPO
FROM --platform=$BUILDPLATFORM $DOCKER_REPO/arena-autocam/builder_base:latest AS builder_base
WORKDIR /root

FROM builder_base AS app_dependencies_builder
ARG TARGETARCH
COPY include/install_dev_dependencies.sh ./install_dev_dependencies.sh
RUN --mount=type=cache,target=/var/cache/apt,sharing=shared \
    --mount=type=cache,target=/var/lib/apt,sharing=shared \
    dpkg --add-architecture $TARGETARCH && \
    apt-get update; \
    chmod +x ./install_dev_dependencies.sh && \
    ./install_dev_dependencies.sh
