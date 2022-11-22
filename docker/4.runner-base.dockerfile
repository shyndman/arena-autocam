# syntax=docker/dockerfile:1

# A base image for running binaries. This image will be architecture dependent,
# and will require the use of qemu emulation when built on a host arch that
# differs from the target arch.

# Depending on the architecture, this value will be one of:
# amd64:   balenalib/raspberrypi4-64-debian:bookworm-run
# aarch64: balenalib/amd64-debian:bookworm-build
ARG DOCKER_BASE_RUN_IMAGE
FROM $DOCKER_BASE_RUN_IMAGE AS runner_base

ARG DOCKER_TARGET_ARCH
COPY include/install_runtime_dependencies.sh ./install_runtime_dependencies.sh
COPY include/install_runtime_dependencies.${DOCKER_TARGET_ARCH}.sh ./install_runtime_dependencies.$DOCKER_TARGET_ARCH.sh
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    chmod +x ./install_runtime_dependencies.sh ./install_runtime_dependencies.$DOCKER_TARGET_ARCH.sh && \
    apt-get update && \
    ./install_runtime_dependencies.sh && \
    ./install_runtime_dependencies.$DOCKER_TARGET_ARCH.sh
