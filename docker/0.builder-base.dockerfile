# syntax=docker/dockerfile:1
ARG DOCKER_REPO
FROM --platform=$BUILDPLATFORM $DOCKER_REPO/balenalib/amd64-debian:bookworm-build AS machine_base
WORKDIR /root

# ~~~~ UPDATE APT AND INSTALL SOME BASICS
RUN --mount=type=cache,target=/var/cache/apt,sharing=shared \
    --mount=type=cache,target=/var/lib/apt,sharing=shared \
    apt-get update; \
    apt-get install \
        bash \
        ca-certificates \
        curl \
        gnupg \
        libclang-dev \
        libssl-dev \
        lsb-release \
        software-properties-common \
        wget \
        xxd;

# ~~~~ ZSH SHELL SETUP
FROM machine_base AS shell_setup
COPY include/install_shell.sh install_shell.sh
COPY include/.p10k.zsh .p10k.zsh
RUN --mount=type=cache,target=/var/cache/apt,sharing=shared \
    --mount=type=cache,target=/var/lib/apt,sharing=shared \
    chmod +x install_shell.sh && \
    ./install_shell.sh \
        -t 'https://github.com/romkatv/powerlevel10k' \
        -p 'https://github.com/zsh-users/zsh-syntax-highlighting.git' \
        -a '[[ ! -f ~/.p10k.zsh ]] || source ~/.p10k.zsh'

# ~~~~ RUST
FROM shell_setup AS rustup_builder
RUN curl https://sh.rustup.rs -sSf > rust_init.sh; \
    chmod +x rust_init.sh;

# Install rust tooling with an x86_64 toolchain (arch-specific targets may be
# added later)
RUN ["/bin/zsh", "-c", "\
    ./rust_init.sh -y \
          --default-host=x86_64-unknown-linux-gnu \
          --default-toolchain=nightly; \
    source '.cargo/env';"]
