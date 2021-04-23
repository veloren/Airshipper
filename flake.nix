{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nixCargoIntegration = {
      url = "github:yusdacra/nix-cargo-integration";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    let
      outputz = inputs.nixCargoIntegration.lib.makeOutputs {
        root = ./.;
      };
    in
    outputz // {
      # Make airshipper client the default package and app
      defaultPackage = builtins.mapAttrs (_: v: v.airshipper) outputz.packages;
      defaultApp = builtins.mapAttrs (_: v: v.airshipper) outputz.apps;
    };
}
