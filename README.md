# Arena Autocam

This project aims to drive a Raspberry Pi 4 camera to smoothly follow a horse
and rider during video capture.

## Requirements

* Git LFS
* Rust nightly toolchain
* dotenv-cli (`npm install --global dotenv-cli`)
* docker-ce docker-ce-cli containerd.io docker-compose-plugin
  * Docker must be configured for cross-platform builds. [These
    instructions](https://medium.com/@artur.klauser/building-multi-architecture-docker-images-with-buildx-27d80f7e2408)
    will help. [A
    script](https://medium.com/@artur.klauser/building-multi-architecture-docker-images-with-buildx-27d80f7e2408#8e70)
    included on that page can be used to verify that the installation is
    correct.

## Building the application

The `tasks` directory contains a custom `cargo` subcommand, "`tasks`", for
performing various development tasks.

Building one of the application's binary targets has two steps.

TODO(shyndman): Describe [Docker multistage
builds](https://docs.docker.com/build/building/multi-stage/), and how we
use the concept of builder images, runner images, and the base images of both.
