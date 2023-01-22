#!/bin/bash
echo "cleaning podman"
podman stop reywen-txc
podman rm reywen-txc

echo "cleaning buildah"
buildah rm fedora-minimal-working-container
buildah rmi localhost/reywen-txc
buildah rm fedora-minimal-working-container

rm txcreywen_latest.tar.gz

echo "done"