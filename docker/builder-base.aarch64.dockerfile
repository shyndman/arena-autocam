# syntax=docker/dockerfile:1
ARG DOCKER_REPO
FROM --platform=$BUILDPLATFORM $DOCKER_REPO/arena-autocam/builder_base:latest AS builder_base
WORKDIR /root

# Update the package manager and install some basics
RUN --mount=type=cache,target=/var/cache/apt,sharing=shared \
    --mount=type=cache,target=/var/lib/apt,sharing=shared \
    apt-get update && \
    apt-get --assume-yes --no-install-recommends install \
        g++-aarch64-linux-gnu \
        libc6-dev-arm64-cross

FROM builder_base AS app_dependencies_builder
ARG TARGETARCH
RUN dpkg --add-architecture $TARGETARCH
COPY include/install_dev_dependencies.sh ./install_dev_dependencies.sh
RUN chmod +x ./install_dev_dependencies.sh
RUN --mount=type=cache,target=/var/cache/apt,sharing=shared \
    --mount=type=cache,target=/var/lib/apt,sharing=shared \
    ./install_dev_dependencies.sh

# Rust target setup
FROM app_dependencies_builder AS rustup_target_builder
RUN ["/bin/zsh", "-c", "\
    rustup target add aarch64-unknown-linux-gnu"]
COPY include/rust_target_env.aarch64.sh /root/rust_target_env.sh
RUN cat ./rust_target_env.sh >> ./.cargo/env && \
    rm ./rust_target_env.sh
