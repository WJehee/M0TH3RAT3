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

