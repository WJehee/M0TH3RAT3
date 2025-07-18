dir := `pwd`

_default:
    just --list

# Build
build:
    cargo build

# Run
run *args:
    cargo run {{args}}

# Run within cool retro terminal
cool-run *args:
    cool-retro-term --workdir {{dir}} --profile "Futuristic" -e ~/Code/M0TH3RAT3/target/debug/M0TH3RAT3 {{args}}

# Cross shell
cross-shell:
    nix-shell -p cargo rustup cargo-cross

# Build binary for ARM, must be in cross shell
build-arm:
    cross build --target armv7-unknown-linux-gnueabihf

