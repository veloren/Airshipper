#!/bin/bash
# NOTE: Run `build_server.ps1` beforehand!
$default = "latest"
if (!($tag = Read-Host "Docker Image Tag [$default]")) { $tag = $default }

docker push docker.pkg.github.com/songtronix/airshipper/airshipper:$tag