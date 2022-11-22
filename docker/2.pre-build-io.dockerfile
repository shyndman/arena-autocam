# syntax=docker/dockerfile:1

# This image is created in the context of a build, and precedes the image that
# builds a specific binary.
#
# It is rebuilt each time a build is requested, ideally performing very little
# work due to caching. It runs in the context of the workspace root.

ARG DOCKER_REPO
ARG DOCKER_TARGET_ARCH
FROM --platform=$BUILDPLATFORM $DOCKER_REPO/arena-autocam/builder_base_${DOCKER_TARGET_ARCH}:latest AS builder_base
WORKDIR /root/build

# Copy all application files not covered by the root .dockerignore
FROM builder_base AS source_copier
COPY . .

# Run a fetch across all targets
FROM source_copier AS dependency_fetcher
RUN --mount=type=cache,target=/root/.cargo/registry,sharing=shared \
    --mount=type=cache,target=/root/.cargo/git,sharing=shared \
    zsh -c "cargo -v fetch"
