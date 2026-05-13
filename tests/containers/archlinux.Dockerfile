FROM docker.io/library/archlinux:latest
RUN pacman -Syu --noconfirm && pacman -S --noconfirm \
    systemd \
    iproute2 \
    procps-ng \
    which \
    base-devel \
    rustup \
    git \
    && pacman -Scc --noconfirm

RUN rustup default stable

WORKDIR /ao
VOLUME /ao
