# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

M0TH3R@3 is a Rust TUI (Terminal User Interface) application for managing a spaceship in the Mothership TTRPG. Built with Ratatui, it provides a retro terminal aesthetic optimized for `cool-retro-term`.

## Build Commands

```bash
just build          # Build with cargo
just run            # Run debug binary
just cool-run       # Run in cool-retro-term with Futuristic profile
cargo run -- path.json  # Run with custom storage file
```

### Cross-compilation (ARM)
```bash
just cross-shell    # Enter nix-shell with cross tools
just build-arm      # Build for armv7-unknown-linux-gnueabihf
```

### Development Environment
```bash
nix-shell           # Provides cargo, rustup, rust-analyzer, clippy, rustfmt
```

## Architecture

### Module Structure
- **main.rs** - Entry point, panic/error hooks that restore terminal state
- **app.rs** - Main game loop, UI rendering, input handling
- **tui.rs** - Terminal initialization/cleanup (raw mode, alternate screen)
- **storage.rs** - JSON persistence for users, planets, components
- **login.rs** - Password-protected login screen with effects
- **user.rs** - Player data (position, resources: fuel, crystals, reputation)
- **objects.rs** - Game world objects (SolarSystem, Planet)
- **components/** - Reusable UI widgets:
  - `galaxy_map.rs` - 2D galactic navigation
  - `star_map.rs` - Solar system view
  - `resources.rs` - Resource gauges
  - `crew.rs` - Crew display with ASCII art

### Game Screens (MenuItem enum)
1. **GalacticMap** - Navigate between solar systems (WASD, uses fuel)
2. **StarMap** - Select planets within a system (Arrow keys)
3. **Crew** - View crew member status

### Key Mechanics
- Warp travel: Hold Enter for 1 second, consumes 1 fuel
- Planet exploration: 'e' key to collect resources
- Events: 'q' to dismiss random planet events

### Data Persistence
- JSON storage (default: `default.json`, fallback: `fallback.json`)
- Stores: users, solar systems with planets, shared component pool

## UI Framework Notes

- Widgets implement Ratatui's `Widget` trait
- Effects via tachyonfx (coalescing animations)
- Green terminal theme with colored resource gauges (magenta, red, yellow, gray)
- Dutch language used for in-game text
