#!/bin/bash

# Used in Docker build to set platform dependent variables.

case $TARGETARCH in
    "amd64")
        echo "x86_64-unknown-linux-gnu" > /app/.platform
        echo "" > /app/.compiler
        ;;
    "arm")
        echo "armv7-unknown-linux-gnueabihf" > /app/.platform
        echo "g++-arm-linux-gnueabihf" > /app/.compiler
        ;;
esac
