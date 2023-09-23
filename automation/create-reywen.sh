#!/bin/bash
echo "compiling reywen"


echo "creating template"
buildah rm fedora-minimal-working-container
buildah from fedora-minimal:latest

echo "compiling"
time mold --run cargo b -r
echo "making directories"
buildah run fedora-minimal-working-container mkdir /server /config
echo "copying files"
buildah copy fedora-minimal-working-container target/release/reywen-txc /server/reywen-txc
buildah copy fedora-minimal-working-container config /config
echo "chmod"
buildah run fedora-minimal-working-container chmod 777 -R /server/reywen-txc
buildah run fedora-minimal-working-container chmod 777 -R /config

echo "creating image"
buildah config --entrypoint "/server/reywen-txc -D FOREGROUND" fedora-minimal-working-container
buildah commit fedora-minimal-working-container reywen-txc