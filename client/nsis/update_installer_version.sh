#!/bin/bash
# Extract the client version from client/Cargo.toml.
# The first grep finds the line, the second grep extracts the version
AIRSHIPPER_VERSION=$(grep -Ei "^version = \".*\"" client/Cargo.toml | grep -oEi "[0-9]+\.[0-9]+\.[0-9]+")

# Replace the version in the installer.nsi file with the version extracted from Cargo.toml
sed -i -E "s/(\!define VERSION \")(.*)(\".*)/\1$AIRSHIPPER_VERSION.0\3/" client/nsis/installer.nsi