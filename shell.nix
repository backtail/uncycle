let
  rustOverlay = import (fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  nixpkgs = fetchTarball "https://github.com/NixOS/nixpkgs/archive/refs/tags/25.05.tar.gz";
  pkgs = import nixpkgs { overlays = [ rustOverlay ]; };

  rustVersion = "1.88.0";
  rustToolchain = pkgs.rust-bin.stable.${rustVersion}.default.override {
    extensions = [ "rust-src" ];
  };

in

pkgs.mkShell {
  name = "uncycle-dev";

  buildInputs = with pkgs; [
    rustToolchain
    rust-analyzer

    pkg-config
    alsa-lib

    stdenv.cc.libc
    stdenv.cc
  ];

  shellHook = ''
    export TMPDIR="/tmp/nix-shell-$$"
    mkdir -p "$TMPDIR"

    export RUST_ANALYZER="${pkgs.rust-analyzer}/bin/rust-analyzer"

    export LD_LIBRARY_PATH="${pkgs.stdenv.cc.libc}/lib:${pkgs.alsa-lib}/lib:$LD_LIBRARY_PATH"
    export PKG_CONFIG_PATH="${pkgs.alsa-lib}/lib/pkgconfig:$PKG_CONFIG_PATH"

    echo "Rust: $(rustc --version)"
    echo "Cargo: $(cargo --version)"
    echo ""
    echo "All tools pinned to nixpkgs 25.05"
  '';

  RUST_BACKTRACE = "full";
  RUST_LOG = "debug";
}
