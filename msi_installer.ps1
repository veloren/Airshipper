# This script will create the .msi installer and yes it's stupid.
$error.clear();
$ErrorActionPreference = "Stop"
cargo build --bin airshipper --release
if (!$error) {
    cargo wix --no-build --nocapture -n client --install-version 0.2.0 -o .packages/
} else {
    Write-Error "Building release failed!"
}