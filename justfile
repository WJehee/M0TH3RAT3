dir := `pwd`

_default:
    just --list

# Build every crate in the workspace
build:
    cargo build --workspace

# Run the Mothership ship interface (pass a storage json path as argument)
run *args:
    cargo run -p mothership -- {{args}}

# Serve the Mothership interface over SSH on port 2222 (RUST_LOG=debug for more detail)
serve *args:
    RUST_LOG=${RUST_LOG:-info} cargo run -p mothership -- --serve {{args}}

# Run the in-car display
car *args:
    cargo run -p car -- {{args}}

# Run the Mothership interface within cool retro terminal
cool-run *args:
    cool-retro-term --workdir {{dir}} --profile "Futuristic" -e {{dir}}/target/debug/mothership {{args}}

# Run the car display within cool retro terminal
cool-car *args:
    cool-retro-term --workdir {{dir}} --profile "Futuristic" -e {{dir}}/target/debug/car {{args}}

# Lint every crate
lint:
    cargo clippy --workspace

# Cross shell
cross-shell:
    nix-shell -p cargo rustup cargo-cross

# Build both binaries for ARM, must be in cross shell
build-arm:
    cross build --workspace --target armv7-unknown-linux-gnueabihf
