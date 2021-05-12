# Flake's devShell for non-flake-enabled nix instances
(import
  (
    let lock = builtins.fromJSON (builtins.readFile ./flake.lock);
    in
    fetchTarball {
      url =
        "https://github.com/edolstra/flake-compat/archive/${lock.nodes.flakeCompat.locked.rev}.tar.gz";
      sha256 = lock.nodes.flakeCompat.locked.narHash;
    }
  )
  { src = ./.; }).shellNix.default
