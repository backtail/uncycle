# uncycle

*A real-time MIDI tool that extends existing hardware synth capabilities.*

## Project Vision

`uncycle` aims to be an open-source, cross-platform MIDI swiss-knife designed to augment your existing hardware instruments rather than replace them. Born from the philosophy of "anti-anti-consumerism," this project helps you get more creative mileage from your current devices.

## TUI

TODO: insert screenshot

### Build

**Using `nix` (Recommended):**

[Install nix](https://nix.dev/install-nix.html), the package manager, on your system. You don't have to install Rust at all with this method. Entering the dev shell might take some time the very first time.
```bash
nix-build tui.nix
```

**Using `cargo`:**

[Install Rust](https://rust-lang.org/tools/install/) and make sure to use at least version 1.88.0.

```bash
cargo run --release
```


## Core Library

Seperating the internal MIDI and business logic from the TUI is planned soon. It will become a `no_std` crate and provide a `cbindgen` API for crosscompiling on embedded platforms with C.

## Hardware

The long-term plan is to run this software on dedicated hardware, i.e. a small stompbox, so that a true DAW-less or PC-less setup can be achieved. Developing in a TUI is a lot less work to get going, though.

## Project Structure

```shell
uncycle/
├── core/   # (planned) platform-agnostic logic
├── fw/     # (planned) firmware for embedded device
└── tui/    # frontend and backend for PC use
```