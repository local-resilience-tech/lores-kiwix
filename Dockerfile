# syntax=docker/dockerfile:1

# Build stage. Runs on the host platform so cross-compilation is fast; the
# resulting binary is for the target platform selected by --platform.
FROM --platform=$BUILDPLATFORM ubuntu:26.04 AS builder

ARG TARGETARCH
ARG TARGETVARIANT

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        curl \
        pkg-config \
        protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Install Rust via rustup and verify cargo is available in the next layer.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
ENV PATH="/root/.cargo/bin:${PATH}"
RUN cargo --version

WORKDIR /app

# Select the Rust target triple and cross-compiler package for this platform.
COPY --chmod=555 deployment/platform.sh .
RUN ./platform.sh

RUN rustup target add $(cat .platform)

# Install libkiwix for the target architecture. For cross-compiled arches
# (arm/v7, arm64) the target libraries come from ports.ubuntu.com and are
# extracted rather than configured, since their maintainer scripts would run
# target-arch binaries that cannot execute on the amd64 build host. For amd64
# we build natively. platform.sh has already written .dpkgarch, .compiler and
# .buildenv for the selected TARGETARCH.
RUN --mount=type=cache,target=/var/cache/apt,id=apt-$TARGETARCH$TARGETVARIANT,sharing=locked \
    DPKG_ARCH="$(cat .dpkgarch)"; \
    if [ -n "$DPKG_ARCH" ]; then \
        dpkg --add-architecture "$DPKG_ARCH" \
        && CODENAME="$(. /etc/os-release && echo "$VERSION_CODENAME")" \
        && printf 'Types: deb\nURIs: http://archive.ubuntu.com/ubuntu\nSuites: %s %s-updates %s-backports\nComponents: main restricted universe multiverse\nArchitectures: amd64\nSigned-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg\n\nTypes: deb\nURIs: http://security.ubuntu.com/ubuntu\nSuites: %s-security\nComponents: main restricted universe multiverse\nArchitectures: amd64\nSigned-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg\n\nTypes: deb\nURIs: http://ports.ubuntu.com/ubuntu-ports\nSuites: %s %s-updates %s-backports %s-security\nComponents: main restricted universe multiverse\nArchitectures: %s\nSigned-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg\n' \
            "$CODENAME" "$CODENAME" "$CODENAME" "$CODENAME" "$CODENAME" "$CODENAME" "$CODENAME" "$CODENAME" "$DPKG_ARCH" \
            > /etc/apt/sources.list.d/ubuntu.sources \
        && apt-get update \
        && apt-get install -y --no-install-recommends $(cat .compiler) \
        && apt-get install -y --no-install-recommends --download-only libkiwix-dev:"$DPKG_ARCH" \
        && for deb in /var/cache/apt/archives/*.deb; do \
            [ -e "$deb" ] || continue; \
            if [ "$(dpkg-deb -f "$deb" Architecture)" = "$DPKG_ARCH" ]; then \
                dpkg-deb -x "$deb" /; \
            fi; \
        done \
        && rm -rf /var/lib/apt/lists/*; \
    else \
        apt-get update \
        && apt-get install -y --no-install-recommends libkiwix-dev \
        && rm -rf /var/lib/apt/lists/*; \
    fi

COPY deployment/cargo-config.toml ./.cargo/config.toml

COPY . .

RUN . /app/.buildenv \
    && cargo build --release --manifest-path crates/lores-kiwix/Cargo.toml --target $(cat /app/.platform) \
    && cp /app/target/$(cat /app/.platform)/release/lores-kiwix /app/lores-kiwix-bin

# Runtime stage
FROM ubuntu:26.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        libkiwix14 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/lores-kiwix-bin /usr/local/bin/lores-kiwix
COPY --from=builder /app/crates/lores-kiwix/static /usr/local/share/lores-kiwix/static

# Configuration environment variables:
#   PANDA_GRPC_ADDR        Address of the lores-node gRPC endpoint.
#   LORES_APP_ID           Application identifier for p2panda topics.
#   LORES_INSTANCE_ID      Instance identifier for p2panda topics.
#   DATA_DIR               Directory for the operations SQLite database.
#   KIWIX_INTERNAL_BIND    Bind address for the internal libkiwix HTTP server.
#   LORES_KIWIX_STATIC_DIR Directory containing static override assets.
#
# ZIM files are provided at runtime by mounting a host directory:
#   docker run -v /path/to/zims:/zim:ro lores-kiwix /zim 0.0.0.0:8080
ENV DATA_DIR=/data
ENV KIWIX_INTERNAL_BIND=127.0.0.1:18080
ENV LORES_KIWIX_STATIC_DIR=/usr/local/share/lores-kiwix/static

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/lores-kiwix"]
