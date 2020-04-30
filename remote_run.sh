#!/usr/bin/env bash

set -e

TARGET_HOST=$1

TARGET_FOLDER=/tmp/zynq-\$USER

ssh $TARGET_HOST "mkdir -p $TARGET_FOLDER"
rsync openocd/* $TARGET_HOST:$TARGET_FOLDER
rsync result/szl $TARGET_HOST:$TARGET_FOLDER
rsync result/top.bit $TARGET_HOST:$TARGET_FOLDER
ssh $TARGET_HOST "cd $TARGET_FOLDER; openocd -f zc706.cfg -c 'pld load 0 top.bit; load_image szl; resume 0; exit'"
