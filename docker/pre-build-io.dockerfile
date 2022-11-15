# syntax=docker/dockerfile:1

# This is the base image for all builds, and is rebuilt each time a build is
# requested, ideally performing very little work. It runs in the context of the
# workspace root.
ARG DOCKER_REPO
ARG TARGETARCH_DOCKER
FROM --platform=$BUILDPLATFORM $DOCKER_REPO/arena-autocam/builder_base_${TARGETARCH_DOCKER}:latest AS builder_base
WORKDIR /root/build

# Perform a search to force an index download
RUN --mount=type=cache,target=/root/.cargo/registry,sharing=shared \
    --mount=type=cache,target=/root/.cargo/git,sharing=shared \
    ["/bin/zsh", "-c", "\
        cargo install cargo-task \
    "]

# Copy all application files not covered by the root .dockerignore
FROM builder_base AS source_copier
COPY . .

# Run a fetch across all targets
FROM source_copier AS dependency_fetcher
RUN --mount=type=cache,target=/root/.cargo/registry,sharing=shared \
    --mount=type=cache,target=/root/.cargo/git,sharing=shared \
    zsh -c "cargo fetch"
