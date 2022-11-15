# syntax=docker/dockerfile:1

# This should be run with a context of the workspace root
ARG DOCKER_REPO
ARG TARGETARCH_DOCKER
FROM --platform=$BUILDPLATFORM $DOCKER_REPO/arena-autocam/pre_build_io_${TARGETARCH_DOCKER}:latest AS pre_build
WORKDIR /root/build

ARG RUST_BUILD_TARGET
ARG RUST_PROFILE
ARG RUST_TARGET

RUN --mount=type=cache,target=/root/.cargo/git,sharing=shared \
    --mount=type=cache,target=/root/.cargo/registry,sharing=shared \
    --mount=type=cache,target=/root/build/target,sharing=shared \
    zsh -c " \
      cargo build --profile ${RUST_PROFILE} --target ${RUST_TARGET} ${RUST_BUILD_TARGET} \
    "
