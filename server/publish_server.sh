#!/bin/bash
# NOTE: EXECUTE THIS FROM THE WORKSPACE ROOT ONLY
sudo docker build server/ -f server/Dockerfile -t docker.pkg.github.com/songtronix/airshipper/airshipper:latest
sudo docker push docker.pkg.github.com/songtronix/airshipper/airshipper:latest