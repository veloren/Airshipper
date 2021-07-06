use crate::{consts, ClientError, Result};
use std::path::Path;

const PATCH_DRV: &str = r#"
{ parentDir, velorenExe, serverExe, pkgs ? import <nixpkgs> {} }:
let
  runtimeLibs =
    with pkgs;
    ([ libGL libxkbcommon libudev alsaLib vulkan-loader vulkan-extension-layer ]
            ++ (with xorg; [ libxcb libX11 libXcursor libXrandr libXi ]));
in
pkgs.stdenv.mkDerivation {
    name = "veloren-patch";
    src = builtins.path {
      path = parentDir;
      filter = path: type: path == velorenExe || path == serverExe;
    };
    phases = [ "installPhase" "fixupPhase" ];
    nativeBuildInputs = [ pkgs.makeWrapper ];
    installPhase = "mkdir $out && cp $src/* $out";
    fixupPhase = ''
        chmod 755 $out/*

        patchelf --set-interpreter "$(cat $NIX_CC/nix-support/dynamic-linker)" \
                         $out/veloren-voxygen
        patchelf --set-interpreter "$(cat $NIX_CC/nix-support/dynamic-linker)" \
                         $out/veloren-server-cli

        wrapProgram $out/veloren-voxygen --set LD_LIBRARY_PATH \
                         "${pkgs.lib.makeLibraryPath runtimeLibs}"
    '';
}
"#;

const OS_RELEASE: &str = "/etc/os-release";

/// Check if we are on NixOS.
pub fn is_nixos() -> Result<bool> {
    let os_release = Path::new(OS_RELEASE);
    Ok(os_release.exists() && std::fs::read_to_string(os_release)?.contains("ID=nixos"))
}

/// Patches the executable files. Required for NixOS.
///
/// Note: it's synchronous!
pub fn patch_elf(voxygen_file: &Path, server_cli_file: &Path) -> Result<()> {
    log::info!("Patching voxygen and server CLI executable files");

    let parent_dir = voxygen_file
        .parent()
        .expect("no parent dir?")
        .to_string_lossy();

    // Patch the files
    let output = std::process::Command::new("nix-build")
        .args(&[
            "--impure",
            "-E",
            PATCH_DRV,
            "--argstr",
            "velorenExe",
            &voxygen_file.to_string_lossy(),
            "--argstr",
            "serverExe",
            &server_cli_file.to_string_lossy(),
            "--argstr",
            "parentDir",
            &parent_dir,
            "--no-link",
        ])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Return error if nix-build fails
    if !output.status.success() {
        return Err(ClientError::Custom(format!(
            "Failed to patch files with Nix: nix-build output:\nstderr: {}\nstdout: {}",
            stderr, stdout
        )));
    }

    // nix-build returns the built derivation's out path in stdout.
    let store_path = Path::new(stdout.as_ref().trim().trim_end_matches('\n'));

    // Remove the original executables.
    std::fs::remove_file(voxygen_file)?;
    std::fs::remove_file(server_cli_file)?;

    // Link the patched files.
    // They must be linked so that Nix doesn't accidentally garbage collect the store
    // paths patched files depend on!
    std::os::unix::fs::symlink(store_path.join(consts::VOXYGEN_FILE), voxygen_file)?;
    std::os::unix::fs::symlink(
        store_path.join(consts::SERVER_CLI_FILE),
        server_cli_file,
    )?;

    Ok(())
}
