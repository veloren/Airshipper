# This script will create the .msi installer and yes it's stupid.
$error.clear();
$ErrorActionPreference = "Stop"
cargo build --bin airshipper --release
if (!$error) {
    # TODO: Get rid of that and adjust main.wxs
    Move-Item -Path .\target\release\airshipper.exe -Destination .\target\release\veloren.exe -Force
    cargo wix --no-build --nocapture -n client --install-version 0.1.2 -o .packages/
} else {
    Write-Error "Building release failed!"
}