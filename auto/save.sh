#!/bin/bash
echo "exporting..."
docker save localhost/reywen-txc:latest | gzip > txcreywen_latest.tar.gz
echo "done!"