# syntax=docker/dockerfile:1

# Copies the binary produced by the final build step into a lightweight
# run image, and sets it as the image's CMD.

ARG DOCKER_REPO
# The name of the docker image that contains the built binary (the result of
# building an image from `3.build-rust-target.dockerfile`)
ARG DOCKER_BUILD_BIN_IMAGE
# The architecture we're targeting, either amd64 or aarch64
ARG DOCKER_TARGET_ARCH

# Give the binary builder image a name, so we can reference it in a COPY later
FROM $DOCKER_BUILD_BIN_IMAGE AS builder

# Use the runner_base as the starting point, which has the application's runtime
# dependencies installed
FROM $DOCKER_REPO/arena-autocam/runner_base_${DOCKER_TARGET_ARCH}:latest AS runner
WORKDIR /app

# This, along with running the container in privileged mode, ensures that the
# container can access devices (like the camera and GPIO pins)
ENV UDEV=true

# Copy the binary out of the builder image
COPY --from=builder /root/output/run ./run

# Setup the binary to run on container start. Note that the binary is provided
# to the Balena image's entrypoint, which itself performs some necessary
# container setup.
CMD [ "/app/run" ]
