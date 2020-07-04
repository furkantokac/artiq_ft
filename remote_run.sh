#!/usr/bin/env bash

set -e

target_host="rpi-4.m-labs.hk"
impure=0
pure_dir="result"
impure_dir="build"
sshopts=""

while getopts "h:id:o:" opt; do
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
    o)  sshopts=$OPTARG
        ;;
    esac
done

target_folder=/tmp/zynq-$USER

echo "Creating $target_folder..."
ssh $sshopts $target_host "mkdir -p $target_folder"
echo "Copying files..."
rsync -e "ssh $sshopts" openocd/* $target_host:$target_folder
if [ $impure -eq 1 ]; then
    rsync -e "ssh $sshopts" $impure_dir/firmware/armv7-none-eabihf/release/szl $target_host:$target_folder/szl.elf
    rsync -e "ssh $sshopts" $impure_dir/gateware/top.bit $target_host:$target_folder
else
    rsync -e "ssh $sshopts" -Lc $pure_dir/szl.elf $target_host:$target_folder
    rsync -e "ssh $sshopts" -Lc $pure_dir/top.bit $target_host:$target_folder
fi
echo "Programming board..."
ssh $sshopts $target_host "cd $target_folder; openocd -f zc706.cfg -c 'pld load 0 top.bit; load_image szl.elf; resume 0; exit'"
