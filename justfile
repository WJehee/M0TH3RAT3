
_default:
    just --list

# Run the app
run:
    cargo run

# Run cool retro terminal
cool-run:
    cool-retro-term --profile "Monochrome Green" -e ~/Code/M0TH3RAT3/target/debug/M0TH3RAT3

