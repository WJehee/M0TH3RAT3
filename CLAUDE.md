# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

M0TH3R@3 is a Cargo workspace of Rust TUI (Terminal User Interface) applications built with Ratatui, with a retro terminal aesthetic optimized for `cool-retro-term`. A shared widget library feeds two apps: a spaceship interface for Mothership TTRPG players, and an in-car display.

## Build Commands

```bash
just build          # Build every crate
just run            # Run the mothership binary
just run path.json  # Mothership with a custom storage file
just car            # Run the car display
just cool-run       # Mothership in cool-retro-term with Futuristic profile
just serve          # Mothership as an SSH server (--listen, --host-key flags)
just lint           # cargo clippy --workspace
just nix-build      # naersk release build via the flake
just nix-check      # nix flake check, includes the NixOS VM test
```

### Nix
- `flake.nix` builds each binary as its own package with naersk and exports `nixosModules.mothership`.
- `nix/module.nix` is the NixOS module: `services.mothership.{enable,port,listenAddress,openFirewall,initialStorage,logLevel,package}`. State is fixed at `/var/lib/mothership` (systemd `StateDirectory`); the seed save is only installed when no save exists.
- `nix/test.nix` is a NixOS VM test that drives a real ssh client through a pty. Run it with `nix build .#checks.x86_64-linux.mothership-service -L`.
- `Storage::load` overwrites the `path` field with the path it loaded from, so a copied save file writes back to its new location.

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

### Workspace layout
- **crates/widgets** (lib `widgets`) - Everything not tied to a domain:
  - `tui.rs` - Local terminal initialization/cleanup (raw mode, alternate screen)
  - `runtime.rs` - `Program` trait, `run` frame loop generic over backend and `EventSource` (`CrosstermEvents` for local, `ChannelEvents` for remote), `KeyHold` for long-press detection, panic hooks
  - `menu.rs` - Generic `Menu<T>` (cursor vs active item, caller renders lines)
  - `notifications.rs` - `Notifications<T>` queue with per-language `Labels`
  - `gauge.rs` - `LabeledGauge`
  - `diagnostics.rs` - Fake signal/spectrum charts
  - `util.rs` - `center`, `XorShift`
- **crates/serve** (lib `serve`) - SSH transport on russh:
  - `lib.rs` - `Service` trait (authenticate, run_session), `run_server`, `ServerConfig`
  - `handler.rs` - russh `Handler`: pty/shell requests spawn a session thread running `widgets::run`
  - `input.rs` - Raw terminal bytes to `KeyEvent`s (Ctrl-C and Ctrl-D map to Esc)
  - `writer.rs` - `io::Write` buffer per frame, forwarded over the channel by a tokio task
  - `keys.rs` - Ed25519 host key load/generate
- **crates/mothership** (bin `mothership`) - The ship game:
  - `main.rs` - clap CLI; local mode runs the app in this terminal, `--serve` implements `serve::Service` over a `Mutex<Storage>` shared by all sessions
  - `app.rs` - `App` implements `Program`; menu, screens, game events
  - `storage.rs`, `user.rs`, `objects.rs`, `login.rs` - Persistence, player, world, login screen
  - `components/` - `galaxy_map`, `star_map`, `resources`, `crew`, `stock_market`
- **crates/car** (bin `car`) - In-car display skeleton:
  - `app.rs` - `App` implements `Program`; Navigation, Music, Vehicle screens
  - `screens/navigation.rs`, `screens/music.rs` - Demo data with TODOs for real sources

Add new generic widgets to `crates/widgets`; keep domain-specific ones in the app crate that uses them. Widgets take their user-facing strings from the app so languages do not leak into the library.

### Mothership screens (MenuItem enum)
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
- Sessions play on a clone of the storage and write back through `App::apply_to` when they end. In serve mode the map is replaced whole, so the last player to leave wins on planet state (see the TODO in `app.rs`).
- `Storage::try_login` holds the login puzzle and is used by both the login screen and SSH password auth.

### Toolchain note
russh is pinned below 0.63 because the dev shell ships rustc 1.88 and newer russh needs 1.89. Bump both together.

## UI Framework Notes

- Widgets implement Ratatui's `Widget` trait
- Effects via tachyonfx (coalescing animations)
- Green terminal theme with colored resource gauges (magenta, red, yellow, gray)
- Dutch language used for in-game text
