
_default:
    just --list

# Run the app
run *args:
    cargo run {{args}}

# Run cool retro terminal
cool-run *args:
    cool-retro-term --profile "Monochrome Green" -e ~/Code/M0TH3RAT3/target/debug/M0TH3RAT3 {{args}}

