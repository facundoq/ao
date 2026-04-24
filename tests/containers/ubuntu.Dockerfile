FROM docker.io/library/ubuntu:24.04
RUN apt-get update && apt-get install -y \
    systemd \
    iproute2 \
    procps \
    which \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*
