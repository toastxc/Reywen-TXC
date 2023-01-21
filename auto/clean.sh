#!/bin/bash
echo "cleaning docker"
docker stop reywen-txc
docker rm reywen-txc
docker image rm localhost/reywen-txc

echo "cleaning buildah"
buildah rm fedora-minimal-working-container
buildah rmi localhost/reywen-txc
buildah rm fedora-minimal-working-container

rm txcreywen_latest.tar.gz

echo "done"