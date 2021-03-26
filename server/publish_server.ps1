#!/bin/bash
# NOTE: Run `build_server.ps1` beforehand!
$default = "master"
if (!($tag = Read-Host "Docker Image Tag [$default]")) { $tag = $default }

docker push docker.pkg.github.com/songtronix/airshipper/airshipper:$tag