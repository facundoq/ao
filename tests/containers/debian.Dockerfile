FROM docker.io/library/debian:12
RUN apt-get update && apt-get install -y \
    systemd \
    iproute2 \
    procps \
    which \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*
