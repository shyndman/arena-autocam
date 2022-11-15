# syntax=docker/dockerfile:1

# This should be run with a context of the workspace root
ARG DOCKER_REPO
ARG TARGETARCH_DOCKER
FROM --platform=$BUILDPLATFORM $DOCKER_REPO/arena-autocam/builder_base_${TARGETARCH_DOCKER}:latest AS builder_base
WORKDIR /root/build

ARG RUST_EXAMPLE
ARG RUST_PROFILE
ARG RUST_TARGET
RUN ["zsh", "-c", "env"]

COPY . .

RUN --mount=type=cache,target=/root/.cargo/git,sharing=shared \
    --mount=type=cache,target=/root/.cargo/registry,sharing=shared \
    --mount=type=cache,target=/root/build/target,sharing=shared \
    zsh -c " \
      cargo build --profile ${RUST_PROFILE} --target ${RUST_TARGET} --example ${RUST_EXAMPLE} \
    "
