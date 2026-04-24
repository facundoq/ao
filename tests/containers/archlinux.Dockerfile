FROM docker.io/library/archlinux:latest
RUN pacman -Syu --noconfirm && pacman -S --noconfirm \
    systemd \
    iproute2 \
    procps-ng \
    which \
    && pacman -Scc --noconfirm
