#!/bin/bash
domain="toastxc.xyz"
local_dir="./txcreywen_latest.tar.gz"
remote_dir="/mnt/raid/reywen/"
echo "transferring..."
sftp $domain:$remote_dir <<< $'put '$local_dir
echo "done"