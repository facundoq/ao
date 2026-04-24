FROM docker.io/library/fedora:40
RUN dnf install -y \
    systemd \
    iproute \
    procps-ng \
    which \
    && dnf clean all
