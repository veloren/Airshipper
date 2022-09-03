{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flakeCompat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
    nci = {
      url = "github:yusdacra/nix-cargo-integration";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs: let
    lib = inputs.nixpkgs.lib;
    cleanedSrc = builtins.path {
      name = "airshipper-source";
      path = toString ./.;
      filter = path: type:
        lib.all
        (n: builtins.baseNameOf path != n)
        [
          "rust-toolchain"
          "rustfmt.toml"
          "shell.nix"
          "default.nix"
          "flake.nix"
          "flake.lock"
          "TROUBLESHOOTING.md"
          "CONTRIBUTING.md"
          "CHANGELOG.md"
          "CODE_OF_CONDUCT.md"
          "WORKFLOW.md"
          "PACKAGING.md"
        ];
    };
  in
    inputs.nci.lib.makeOutputs {
      root = ./.;
      defaultOutputs = {
        app = "airshipper";
        package = "airshipper";
      };
      overrides.crates = common: _: {
        airshipper = prev: {
          src = cleanedSrc;
        };
        airshipper-server = prev: {
          src = cleanedSrc;
        };
      };
      perCrateOverrides = {
        airshipper.packageMetadata = prev: {
          runtimeLibs = [
            "vulkan-loader"
            "wayland"
            "wayland-protocols"
            "libxkbcommon"
            "xorg.libX11"
            "xorg.libXrandr"
            "xorg.libXi"
            "xorg.libXcursor"
          ];
        };
      };
    };  
}
