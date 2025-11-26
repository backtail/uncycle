# uncycle

*A real-time MIDI tool that extends existing hardware synth capabilities.*

## Disclaimer

The project is currently in VEAS (very early alpha stage).

## Project Vision

`uncycle` aims to be an open-source, cross-platform MIDI swiss-knife designed to augment your existing hardware instruments rather than replace them. Born from the philosophy of "anti-anti-consumerism," this project helps you get more creative mileage from your current devices.

## Supported Devices

| Manufacturer | Name | MIDI monitoring | MIDI augmentation |
| ------------ | ---- | :-------------: | :---------------: |
| Roland       | TR-8 |     mostly      |        TBA        |

## TUI

![test](doc/screenshot_alpha_tr8.png)

### Build

**Using `nix` (Recommended):**

[Install nix](https://nix.dev/install-nix.html), the package manager, on your system. You don't have to install Rust at all with this method. Entering the dev shell might take a little the very first time.
```bash
# without flakes
nix-build 
./result/bin/uncycle-tui

# with flakes
nix run 
```

**Using `cargo`:**

[Install Rust](https://rust-lang.org/tools/install/) and make sure to use at least version 1.88.0.

```bash
cargo run --release
```

## Core Library

The core library has been fully switched over to `no_std` by default and further advancement will require this. At some point (hopefully) I will provide a `cbindgen` API for crosscompiling on embedded platforms with C.

## Hardware

The long-term plan is to run this software on dedicated hardware, i.e. a small stompbox, so that a true DAW-less or PC-less setup can be achieved. Developing in a TUI is a lot less work to get going, though.

## Project Structure

```shell
uncycle/
├── core/   # platform-agnostic logic (`#![no_std]` per default)
├── fw/     # (planned) firmware for embedded device
├── tui/    # frontend and backend for PC use
└── vst/    # maybe, who knows
```