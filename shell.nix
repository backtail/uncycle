let
  rustOverlay = import (fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  nixpkgs = fetchTarball "https://github.com/NixOS/nixpkgs/archive/refs/tags/25.05.tar.gz";
  pkgs = import nixpkgs { overlays = [ rustOverlay ]; };

  rustVersion = "1.80.0";
  rustToolchain = pkgs.rust-bin.stable.${rustVersion}.default.override {
    extensions = [ "rust-src" ];
  };

in

pkgs.mkShell {
  name = "uncycle-dev";

  buildInputs = with pkgs; [
    rustToolchain

    pkg-config
    alsa-lib
  ];

  shellHook = ''
    echo "Rust: $(rustc --version)"
    echo "Cargo: $(cargo --version)"
    echo ""
    echo "All tools pinned to nixpkgs 25.05"
  '';

  RUST_BACKTRACE = "full";
  RUST_LOG = "debug";
}
