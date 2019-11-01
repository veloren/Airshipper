# This script will run all tests
$error.clear();
$ErrorActionPreference = "Stop"

Set-Location client; cargo test -q -- --test-threads=1; Set-Location ..; 
Set-Location server; cargo test -q --features test -- --test-threads=1; Set-Location ..; 