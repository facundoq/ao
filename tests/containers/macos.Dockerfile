FROM sickcodes/docker-osx:auto

# This container runs a full macOS VM via QEMU.
# It is used for integration testing ao on macOS.
# Note: Requires /dev/kvm and significant resources.

# Install build tools and Rust on the host to support in-container development
# Since the base image is Arch Linux, we use pacman.
RUN pacman -Syu --noconfirm && pacman -S --noconfirm \
    base-devel \
    rustup \
    git \
    openssh \
    curl \
    iproute2 \
    procps-ng \
    which \
    && pacman -Scc --noconfirm

RUN rustup default stable

WORKDIR /ao
VOLUME /ao
