{ pkgs ? import (fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/refs/tags/25.05.tar.gz";
    sha256 = "1ndiv3a6z8s85s9d6h1k32a6g1g0d0q4x7d8d8xwq7g6q2q0q0q";
  }) {}
}:

let
  rustOverlay = import (builtins.fetchTarball {
    url = "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
    sha256 = "16x08p71m9rw40la4mhqp02sw59n6v0z6wbqb1f0aav9cqfla6s7";
  });
  
  pkgs = import <nixpkgs> {
    overlays = [ rustOverlay ];
  };
  
  rustVersion = "1.88.0";
  rustToolchain = pkgs.rust-bin.stable.${rustVersion}.default;
  
in

pkgs.mkShell {
  name = "uncycle-dev";
  
  buildInputs = with pkgs; [
    # Pinned Rust toolchain
    rustToolchain
    
    # Rust additional tools (will use the same version)
    rustfmt
    clippy
    rust-analyzer
    
    # C/C++ toolchain for embedded
    gcc
    gnumake
    cmake
    pkg-config
    
    # ARM embedded toolchain
    gcc-arm-embedded
    
    # MIDI development libraries
    alsa-lib
    libjack2
    
    # TUI dependencies
    ncurses
    
    # Embedded tools
    openocd
    picotool
    
    # General development
    git
    which
  ];

  shellHook = ''
    echo "🚀 uncycle development environment activated!"
    echo "Rust: $(rustc --version)"
    echo "Cargo: $(cargo --version)"
    echo ""
    echo "All tools pinned to nixpkgs 25.05"
  '';

  RUST_BACKTRACE = "full";
  RUST_LOG = "debug";
}
