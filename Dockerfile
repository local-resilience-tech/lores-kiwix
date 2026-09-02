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

# Install libkiwix for the target architecture. For ARM/v7 we add the armhf
# architecture and cross-compile; for amd64 we build natively.
RUN --mount=type=cache,target=/var/cache/apt \
    if [ "$TARGETARCH" = "arm" ]; then \
        dpkg --add-architecture armhf \
        && apt-get update \
        && apt-get install -y --no-install-recommends \
            $(cat .compiler) \
            libkiwix-dev:armhf \
        && rm -rf /var/lib/apt/lists/* \
        && printf 'PKG_CONFIG_LIBDIR=/usr/lib/arm-linux-gnueabihf/pkgconfig\nPKG_CONFIG_SYSROOT_DIR=/\nCC=arm-linux-gnueabihf-gcc\nCXX=arm-linux-gnueabihf-g++\n' > /app/.buildenv; \
    else \
        apt-get update \
        && apt-get install -y --no-install-recommends libkiwix-dev \
        && rm -rf /var/lib/apt/lists/* \
        && printf '' > /app/.buildenv; \
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

ENV DATA_DIR=/data
ENV KIWIX_INTERNAL_BIND=127.0.0.1:18080
ENV LORES_KIWIX_STATIC_DIR=/usr/local/share/lores-kiwix/static

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/lores-kiwix"]
