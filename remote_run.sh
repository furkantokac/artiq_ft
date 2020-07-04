#!/usr/bin/env bash

set -e

target_host="rpi-4.m-labs.hk"
impure=0
pure_dir="result"
impure_dir="build"

while getopts "h:id:" opt; do
    case "$opt" in
    \?) exit 1
        ;;
    h)  target_host=$OPTARG
        ;;
    i)  impure=1
        ;;
    d)  pure_dir=$OPTARG;
        impure_dir=$OPTARG;
        ;;
    esac
done

target_folder=/tmp/zynq-\$USER

echo "Creating $target_folder..."
ssh $target_host "mkdir -p $target_folder"
echo "Copying files..."
rsync openocd/* $target_host:$target_folder
if [ $impure -eq 1 ]; then
    rsync $impure_dir/firmware/armv7-none-eabihf/release/szl $target_host:$target_folder/szl.elf
    rsync $impure_dir/gateware/top.bit $target_host:$target_folder
else
    rsync -L $pure_dir/szl.elf $target_host:$target_folder
    rsync -L $pure_dir/top.bit $target_host:$target_folder
fi
echo "Programming board..."
ssh $target_host "cd $target_folder; openocd -f zc706.cfg -c 'pld load 0 top.bit; load_image szl.elf; resume 0; exit'"
