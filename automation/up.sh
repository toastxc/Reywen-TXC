#!/bin/bash
sh automation/create-reywen.sh
sh automation/deploy-pod.sh
sh automation/deploy-mongo.sh
sh automation/deploy-reywen.sh