#!/bin/bash
sudo docker build . -t docker.pkg.github.com/songtronix/airshipper/airshipper:latest
sudo docker push docker.pkg.github.com/songtronix/airshipper/airshipper:latest