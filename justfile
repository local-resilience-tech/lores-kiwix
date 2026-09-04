# Justfile for lores-kiwix development.
#
# Run `just dev` to start the dev gRPC server and two lores-kiwix instances.

# Directory containing ZIM files to serve.
export ZIM_DIR := env_var_or_default("ZIM_DIR", "./data/zims")

# gRPC address shared by all dev-mode processes.
export PANDA_GRPC_ADDR := env_var_or_default("PANDA_GRPC_ADDR", "http://127.0.0.1:50051")

# Base HTTP port for the first lores-kiwix instance; the second uses port+1.
export BASE_PORT := env_var_or_default("BASE_PORT", "8080")
export BASE_PORT_2 := shell("echo $(( " + BASE_PORT + " + 1 ))")
export RUST_LOG := env_var_or_default("RUST_LOG", "info")

# Run the full multi-instance development stack.
dev:
    mkdir -p .dev/data-a/zims .dev/data-b/zims
    mprocs --config mprocs.yaml

clean_ops:
    rm .dev/data-a/operations.sqlite
    rm .dev/data-b/operations.sqlite

# Build all workspace crates.
build:
    cargo build

# Build and run just the dev gRPC server.
run-dev-server:
    cargo run -p lores-kiwix-dev-server

# Build the Docker image and run a container from it.
# ZIM_PATH can be relative (from the justfile directory) or absolute.
docker-run ZIM_PATH=ZIM_DIR PORT="8080":
    docker build -t lores-kiwix .
    docker run --rm -it \
        --init \
        --network host \
        -e PANDA_GRPC_ADDR=http://127.0.0.1:50051 \
        -v "{{absolute_path(ZIM_PATH)}}":/zim:ro \
        --name lores-kiwix \
        lores-kiwix /zim 0.0.0.0:8080

# Stop a running `just docker-run` container.
docker-stop:
    docker stop lores-kiwix

# Install required dev tools (run this once after cloning).
setup:
    cargo install cargo-release

# Dry-run a release (no changes made) — pick: patch, minor, or major.
release-dry level:
    cargo test
    cargo release {{level}} -p lores-kiwix

# Execute a release — bumps version, commits, tags, and pushes.
release level:
    cargo test
    cargo release {{level}} -p lores-kiwix --execute
