# syntax=docker/dockerfile:1
ARG DOCKER_REPO
FROM --platform=$BUILDPLATFORM $DOCKER_REPO/arena-autocam/builder_base:latest AS builder_base
WORKDIR /root

# Update the package manager and install some basics
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update; \
    apt-get install \
        g++-aarch64-linux-gnu \
        libc6-dev-arm64-cross \
        unzip

FROM builder_base AS app_dependencies_builder
ARG TARGETARCH
COPY include/install_dev_dependencies.sh ./install_dev_dependencies.sh
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    dpkg --add-architecture $TARGETARCH; \
    apt-get update; \
    chmod +x ./install_dev_dependencies.sh; \
    ./install_dev_dependencies.sh

FROM app_dependencies_builder AS pigpio_builder
ARG GCC_TOOLCHAIN
RUN wget https://github.com/joan2937/pigpio/archive/master.zip; \
    unzip master.zip; \
    cd pigpio-master; \
    make CROSS_PREFIX=$GCC_TOOLCHAIN-; \
    make prefix=/usr/$GCC_TOOLCHAIN install;

# Rust target setup
FROM pigpio_builder AS rustup_target_builder
RUN ["/bin/zsh", "-c", "\
    rustup target add aarch64-unknown-linux-gnu"]
COPY include/rust_target_env.aarch64.sh /root/rust_target_env.sh
RUN cat ./rust_target_env.sh >> ./.cargo/env && \
    rm ./rust_target_env.sh
