{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flakeCompat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
    nixCargoIntegration = {
      url = "github:yusdacra/nix-cargo-integration";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    inputs.nixCargoIntegration.lib.makeOutputs {
      root = ./.;
      defaultOutputs = {
        app = "airshipper";
        package = "airshipper";
      };
    };
}
