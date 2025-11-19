{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      flake-utils,
    }:

    flake-utils.lib.eachDefaultSystem (system: {
      devShells.default = import ./shell.nix { inherit system; };
      packages.default = import ./default.nix { inherit system; };
    });
}
