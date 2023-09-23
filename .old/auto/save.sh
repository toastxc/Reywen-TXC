#!/bin/bash
echo "exporting..."
podman save localhost/reywen-txc:latest | gzip > txcreywen_latest.tar.gz
echo "done!"