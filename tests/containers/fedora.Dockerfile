FROM docker.io/library/fedora:40
RUN dnf install -y \
    systemd \
    iproute \
    procps-ng \
    which \
    gcc \
    make \
    curl \
    git \
    && dnf clean all

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /ao
VOLUME /ao
