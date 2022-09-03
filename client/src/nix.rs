use crate::{ClientError, Result};
use std::path::Path;

const OS_RELEASE: &str = "/etc/os-release";

/// Get patcher for patching the binaries.
fn get_patcher() -> Option<std::ffi::OsString> {
    std::env::var_os("VELOREN_PATCHER")
}

/// Check if we are on NixOS.
pub fn is_nixos() -> Result<bool> {
    let os_release = Path::new(OS_RELEASE);
    Ok(os_release.exists() && std::fs::read_to_string(os_release)?.contains("ID=nixos"))
}

/// Patches the executable files. Required for NixOS.
///
/// Note: it's synchronous!
pub fn patch(profile_directory: &Path) -> Result<()> {
    tracing::info!("Patching voxygen and server CLI executable files for NixOS");

    let patcher = get_patcher().ok_or_else(|| {
        ClientError::Custom("patcher binary was not detected.".to_string())
    })?;

    // Patch the files
    tracing::info!("Executing {patcher:?} on directory {profile_directory:?}");
    let output = std::process::Command::new(patcher)
        .current_dir(profile_directory)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Return error if patcher fails
    if !output.status.success() {
        return Err(ClientError::Custom(format!(
            "Failed to patch files for NixOS: patcher output:\nstderr: \
             {stderr}\nstdout: {stdout}",
        )));
    } else {
        tracing::info!("Patched voxygen and server CLI executable files:\n{stdout}");
    }

    Ok(())
}
