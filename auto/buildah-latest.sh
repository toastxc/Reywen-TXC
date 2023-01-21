#!/bin/bash
echo "compiling reywen"
cargo b -r

echo "creating template"
buildah rm fedora-minimal-working-container
buildah from fedora-minimal:latest

echo "copying files"
buildah copy fedora-minimal-working-container target/release/reywen-txc reywen-txc
buildah copy fedora-minimal-working-container config /config
echo "chmoding"
buildah run fedora-minimal-working-container chmod 777 -R /reywen-txc
buildah run fedora-minimal-working-container chmod 777 -R /config

echo "creating image"
buildah config --entrypoint "/reywen-txc -D FOREGROUND" fedora-minimal-working-container
buildah commit fedora-minimal-working-container reywen-txc

echo "removing old docker container"
docker container prune --force
docker image rm localhost/reywen-txc:latest

echo "pushing image to docker"
buildah push localhost/reywen-txc:latest docker-daemon:localhost/reywen-txc:latest
echo "done"