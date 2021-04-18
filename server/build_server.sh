#!/bin/bash
# NOTE: EXECUTE THIS FROM THE WORKSPACE ROOT ONLY
cp Cargo.lock server/Cargo.lock
sleep 1
sudo docker build server/ -f server/Dockerfile -t dghcr.io/songtronix/airshipper:master
sleep 1
rm server/Cargo.lock
sleep 1