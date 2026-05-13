FROM docker.io/library/ubuntu:24.04
RUN apt-get update && apt-get install -y \
    systemd \
    iproute2 \
    procps \
    which \
    build-essential \
    curl \
    git \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /ao
VOLUME /ao
