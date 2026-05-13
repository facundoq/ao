FROM docker.io/library/alpine:3.20
RUN apk add --no-cache \
    bash \
    iproute2 \
    procps \
    which \
    curl \
    gcc \
    musl-dev \
    make \
    git

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /ao
VOLUME /ao
