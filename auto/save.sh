#!/bin/bash
echo "exporting..."
rm txcreywen_latest.tar.gz
docker save localhost/reywen-txc:latest | gzip > txcreywen_latest.tar.gz
echo "done!"