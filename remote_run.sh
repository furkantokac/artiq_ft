#!/usr/bin/env bash

set -e
set -m

TARGET_HOST=$1

TARGET_FOLDER=/tmp/zynq-\$USER

ssh $TARGET_HOST "mkdir -p $TARGET_FOLDER"
rsync openocd/* $TARGET_HOST:$TARGET_FOLDER
rsync target/armv7-none-eabihf/release/runtime $TARGET_HOST:$TARGET_FOLDER
ssh -n $TARGET_HOST "cd $TARGET_FOLDER; openocd -f zc706.cfg -c 'load_image runtime; resume 0; exit'"
