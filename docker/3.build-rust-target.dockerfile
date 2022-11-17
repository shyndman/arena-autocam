# syntax=docker/dockerfile:1

# This should be run with a context of the workspace root
ARG DOCKER_REPO
ARG DOCKER_TARGET_ARCH
FROM --platform=$BUILDPLATFORM $DOCKER_REPO/arena-autocam/pre_build_io_${DOCKER_TARGET_ARCH}:latest AS pre_build
WORKDIR /root/build

# The binary/example to build, expressed as a CLI option string (ie,
# --bin=aa-app)
ARG RUST_BUILD_TARGET
# The build profile (dev, release, etc)
ARG RUST_PROFILE
# The target triple
ARG RUST_TARGET
# The build profile (debug, release)
ARG RUST_PROFILE_OUT_DIR
# These are provided optionally to configure pkg-config for cross-arch builds
ARG RUST_PKG_CONFIG_PATH=""
ARG RUST_PKG_CONFIG_ALLOW_CROSS=""

# Build! We cache everything heavily, and copy out artifacts in a future step.
RUN --mount=type=cache,target=/root/.cargo/git,sharing=shared \
    --mount=type=cache,target=/root/.cargo/registry,sharing=shared \
    --mount=type=cache,target=/root/build/target,sharing=shared \
    zsh -c " \
      ${RUST_PKG_CONFIG_PATH} \
      ${RUST_PKG_CONFIG_ALLOW_CROSS} \
      RUST_BACKTRACE=1 \
      cargo -vvv build \
          --profile ${RUST_PROFILE} \
          --target ${RUST_TARGET} \
          ${RUST_BUILD_TARGET}"

FROM pre_build AS extract_artifacts
COPY docker/include/extract_binary_artifact.py ./extract_binary_artifact.py
RUN chmod +x extract_binary_artifact.py
RUN --mount=type=cache,target=/root/build/target,sharing=shared \
    ./extract_binary_artifact.py
