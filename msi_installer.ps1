# This script will create the .msi installer and yes it's stupid.
$error.clear();
$ErrorActionPreference = "Stop"
cargo build --release
if (!$error) {
    Move-Item -Path .\target\release\airshipper.exe -Destination .\target\release\veloren.exe -Force
    cargo wix --no-build --nocapture -o .packages/
} else {
    Write-Error "Building release failed!"
}