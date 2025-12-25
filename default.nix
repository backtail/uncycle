{
  system ? builtins.currentSystem,
}:

let
  buildEnv = import ./dependencies.nix { inherit system; };
in

buildEnv.pkgs.rustPlatform.buildRustPackage {

  pname = buildEnv.cargoToml.package.name;
  version = buildEnv.cargoToml.package.version;

  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;

  nativeBuildInputs = buildEnv.nativePlatformDeps ++ [ buildEnv.rustToolchain ];
  buildInputs = buildEnv.platformDeps;

  cargoBuildFlags = [
    "-p"
    buildEnv.cargoToml.package.name
  ];

  doCheck = true;

  meta = with buildEnv.pkgs.lib; {
    mainProgram = buildEnv.cargoToml.package.name;
    description = buildEnv.cargoToml.package.description;
    license = licenses.gpl3Only;
  };
}
