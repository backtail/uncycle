let
  rustOverlay = import (fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  nixpkgs = fetchTarball "https://github.com/NixOS/nixpkgs/archive/refs/tags/25.05.tar.gz";

  pkgs = import nixpkgs {
    system = builtins.currentSystem;
    overlays = [ rustOverlay ];
  };

  rustVersion = "1.88.0";
  rustToolchain = pkgs.rust-bin.stable.${rustVersion}.default.override {
    extensions = [ "rust-src" ];
  };

  platformDeps =
    with pkgs;
    if stdenv.isDarwin then
      [
        darwin.apple_sdk.frameworks.CoreFoundation
        darwin.apple_sdk.frameworks.CoreAudio
        darwin.apple_sdk.frameworks.AudioToolbox
      ]
    else
      [
        alsa-lib
        pkg-config
      ];

in

pkgs.mkShell {
  name = "uncycle-dev";

  buildInputs =
    with pkgs;
    [
      rustToolchain
      clang
    ]
    ++ platformDeps;

  shellHook = ''
    export TMPDIR="/tmp/nix-shell-$$"
    mkdir -p "$TMPDIR"

    echo "Platform: ${pkgs.stdenv.system}"
    echo "Rust: $(rustc --version)"
    echo "Cargo: $(cargo --version)"
    echo ""
    echo "All tools pinned to nixpkgs 25.05"
  '';

  RUST_BACKTRACE = "full";
  RUST_LOG = "debug";
}
