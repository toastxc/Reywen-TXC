#!/bin/bash
podman run -d \
--name reywen-mongodb \
--pod reywen-pod \
-e MONGO_INITDB_ROOT_USERNAME=username \
-e MONGO_INITDB_ROOT_PASSWORD=password \
mongo:latest
