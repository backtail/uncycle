{
  pkgs ? import <nixpkgs> { },
}:

let
  cargoToml = builtins.fromTOML (builtins.readFile ./tui/Cargo.toml);
  package = cargoToml.package;
in

pkgs.rustPlatform.buildRustPackage {
  pname = package.name;
  version = package.version;

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  nativeBuildInputs = with pkgs; [
    pkg-config
  ];

  buildInputs = with pkgs; [
    alsa-lib
  ];
}
