#!/bin/bash

# Used in Docker build to set platform dependent variables.
#
# Writes the following files consumed by the Dockerfile:
#   .platform   Rust target triple.
#   .compiler   apt package(s) for the cross-compiler ("" when building natively).
#   .dpkgarch   dpkg foreign architecture to add ("" when building natively).
#   .buildenv   env vars sourced before `cargo build` ("" when building natively).

case $TARGETARCH in
    "amd64")
        echo "x86_64-unknown-linux-gnu" > /app/.platform
        echo "" > /app/.compiler
        echo "" > /app/.dpkgarch
        printf '' > /app/.buildenv
        ;;
    "arm")
        echo "armv7-unknown-linux-gnueabihf" > /app/.platform
        echo "g++-arm-linux-gnueabihf" > /app/.compiler
        echo "armhf" > /app/.dpkgarch
        printf 'export PKG_CONFIG_ALLOW_CROSS=1\nexport PKG_CONFIG_LIBDIR=/usr/lib/arm-linux-gnueabihf/pkgconfig\nexport PKG_CONFIG_SYSROOT_DIR=/\nexport TARGET_CC=arm-linux-gnueabihf-gcc\nexport TARGET_CXX=arm-linux-gnueabihf-g++\n' > /app/.buildenv
        ;;
    "arm64")
        echo "aarch64-unknown-linux-gnu" > /app/.platform
        echo "g++-aarch64-linux-gnu" > /app/.compiler
        echo "arm64" > /app/.dpkgarch
        printf 'export PKG_CONFIG_ALLOW_CROSS=1\nexport PKG_CONFIG_LIBDIR=/usr/lib/aarch64-linux-gnu/pkgconfig\nexport PKG_CONFIG_SYSROOT_DIR=/\nexport TARGET_CC=aarch64-linux-gnu-gcc\nexport TARGET_CXX=aarch64-linux-gnu-g++\n' > /app/.buildenv
        ;;
esac
