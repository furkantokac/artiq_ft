#!/usr/bin/env bash

set -e

target_host="rpi-4.m-labs.hk"
impure=0

while getopts "h:i" opt; do
    case "$opt" in
    \?) exit 0
        ;;
    h)  target_host=$OPTARG
        ;;
    i)  impure=1
        ;;
    esac
done

target_folder=/tmp/zynq-\$USER

ssh $target_host "mkdir -p $target_folder"
rsync openocd/* $target_host:$target_folder
if [ $impure -eq 1 ]; then
    rsync src/target/armv7-none-eabihf/release/szl $target_host:$target_folder
    rsync src/build/top.bit $target_host:$target_folder
else
    rsync result/szl $target_host:$target_folder
    rsync result/top.bit $target_host:$target_folder
fi
ssh $target_host "cd $target_folder; openocd -f zc706.cfg -c 'pld load 0 top.bit; load_image szl; resume 0; exit'"
