# uncycle

> **Break the upgrade cycle** - A modular MIDI looper that extends the life of your existing gear

## Project Vision

`uncycle` is an open-source, cross-platform MIDI looper designed to augment your existing hardware instruments rather than replace them. Born from the philosophy of "anti-anti-consumerism," this project helps you get more creative mileage from your current setup without constantly chasing the next hardware upgrade.

## Project Structure

```shell
uncycle/
├── Cargo.toml              # Workspace manifest
├── shell.nix               # Nix development environment
├── lib/                    # Core MIDI looper engine
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs          # (Planned) Core library
└── tui/                    # Terminal User Interface
    ├── Cargo.toml
    └── src/
        ├── main.rs         # Application entry point
        ├── keybindings.rs  # Key binding configuration
        └── state.rs        # Application state management
```


## Current Status: **Phase 1 - TUI Development**

## Layered Architecture

```
┌─────────────────┐
│    Frontend     │  (TUI, GUI, Embedded UI)
│  (Application)  │
└─────────────────┘
         │
┌─────────────────┐
│   Core API      │  (C-compatible, thin wrapper)
│  (uncycle.h)    │
└─────────────────┘
         │
┌─────────────────┐
│  Engine Layer   │  (Pure Rust business logic)
│ (State Machine) │
└─────────────────┘
         │
┌─────────────────┐
│  MIDI Layer     │  (Platform-specific backends)
│   (Backends)    │
└─────────────────┘
```
### Key Bindings
- `R` - Start recording
- `S` - Stop recording/playback  
- `P` - Start playback
- `Space` - Toggle play/stop
- `C` - Clear loop count
- `Q` - Quit application

### Development Setup

**Using Nix (Recommended):**
```bash
nix-shell
cargo run --package uncycle-tui
```
