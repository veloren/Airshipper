//! Deals with all filesystem specific details

use crate::{consts, error::ClientError, profiles::Profile, Result};
use std::path::{Path, PathBuf};

lazy_static::lazy_static! {
    // Base for config, profiles, ...
    static ref BASE_PATH: PathBuf = base();
}

// TODO: Is there a way to figure out whether airshipper has been installed or not
//       to allow to use another base location and avoid polluting the current install
// while developing?

/// Returns the base path where all airshipper files like config, profiles belong.
///
/// |Platform | Example                                                       |
/// | ------- | ------------------------------------------------------------- |
/// | Linux   | /home/alice/.local/share/barapp                               |
/// | macOS   | /Users/Alice/Library/Application Support/com.Foo-Corp.Bar-App |
/// | Windows | C:\Users\Alice\AppData\Roaming                                |
fn base() -> PathBuf {
    let path = dirs::data_dir()
        .expect("Couldn't locate where to put launcher data!")
        .join("airshipper");
    std::fs::create_dir_all(&path).expect("failed to create data directory!");
    path
}

pub fn base_path() -> impl std::fmt::Display {
    BASE_PATH.display()
}

#[cfg(windows)]
pub fn get_cache_path() -> PathBuf {
    dirs::cache_dir().unwrap().join(env!("CARGO_PKG_NAME"))
}

/// Returns path to the file which saves the current state
pub fn savedstate_file() -> PathBuf {
    BASE_PATH.join(consts::SAVED_STATE_FILE)
}

/// Returns path to a profile while creating the folder
pub fn profile_path(profile_name: &str) -> PathBuf {
    let path = BASE_PATH.join("profiles").join(profile_name);
    std::fs::create_dir_all(&path).expect("failed to profile directory!"); // TODO
    path
}

/// Returns path to the file where the logs will be stored
pub fn log_file() -> PathBuf {
    BASE_PATH.join(consts::LOG_FILE)
}

/// Extracts downloaded zip file and deletes it after successful extraction.
///
/// Note: it's synchronous!
pub fn unzip(profile: &Profile) -> Result<()> {
    log::info!("Unzipping to {:?}", profile.directory);
    let mut zip_file =
        std::fs::File::open(&profile.directory.join(consts::DOWNLOAD_FILE))?;

    let mut archive = zip::ZipArchive::new(&mut zip_file)?;

    // Delete all assets to ensure that no obsolete assets will remain.
    if profile.directory.join("assets").exists() {
        std::fs::remove_dir_all(profile.directory.join("assets"))?;
    }

    for i in 1..archive.len() {
        let mut file = archive.by_index(i)?;
        // TODO: Verify that `sanitized_name()` works correctly in this case.
        #[allow(deprecated)]
        let path = profile.directory.join(file.sanitized_name());

        if file.is_dir() {
            std::fs::create_dir_all(path)?;
        } else {
            let mut target = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path)?;

            std::io::copy(&mut file, &mut target)?;
        }
    }

    // Delete downloaded zip
    log::trace!("Extracted files, deleting zip archive.");
    std::fs::remove_file(profile.directory.join(consts::DOWNLOAD_FILE))?;

    Ok(())
}

#[cfg(unix)]
const PATCH_DRV: &str = "
{ parentDir, velorenExe, serverExe, pkgs ? import <nixpkgs> {} }:
let runtimeLibs = with pkgs; ([ libGL libxkbcommon libudev alsaLib ] ++ (with xorg; [ \
                         libxcb libX11 libXcursor libXrandr libXi ])); in
pkgs.stdenv.mkDerivation {
    name = \"veloren-patch\";
    src = builtins.path {
      path = parentDir;
      filter = path: type: path == velorenExe || path == serverExe;
    };
    phases = [ \"installPhase\" \"fixupPhase\" ];
    nativeBuildInputs = [ pkgs.makeWrapper ];
    installPhase = \"mkdir $out && cp $src/* $out\";
    fixupPhase = ''
        chmod 755 $out/*

        patchelf --set-interpreter \"$(cat $NIX_CC/nix-support/dynamic-linker)\" \
                         $out/veloren-voxygen
        patchelf --set-interpreter \"$(cat $NIX_CC/nix-support/dynamic-linker)\" \
                         $out/veloren-server-cli

        wrapProgram $out/veloren-voxygen --set LD_LIBRARY_PATH \
                         \"${pkgs.lib.makeLibraryPath runtimeLibs}\"
    '';
}
";

/// Patches the executable files. Required for NixOS.
///
/// Note: it's synchronous!
#[cfg(unix)]
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
