#!/usr/bin/env bash

set -e

impure=0
load_bitstream=1
board_host="192.168.1.52"

while getopts "ilb:" opt; do
    case "$opt" in
    \?) exit 1
        ;;
    i)  impure=1
        ;;
    l)  load_bitstream=0
        ;;
    b)  board_host=$OPTARG
        ;;
    esac
done

load_bitstream_cmd=""

build_dir=`pwd`/build
result_dir=`pwd`/result
cd $OPENOCD_ZYNQ
if [ $impure -eq 1 ]; then
    if [ $load_bitstream -eq 1 ]; then
        load_bitstream_cmd="-g $build_dir/gateware/top.bit"
    fi
    openocd -f zc706.cfg -c "load_image $build_dir/firmware/armv7-none-eabihf/debug/szl; resume 0; exit"
    sleep 5
    artiq_netboot $load_bitstream_cmd -f $build_dir/runtime.bin -b $board_host
else
    if [ $load_bitstream -eq 1 ]; then
        load_bitstream_cmd="-g $result_dir/top.bit"
    fi
    openocd -f zc706.cfg -c "load_image $result_dir/szl.elf; resume 0; exit"
    sleep 5
    artiq_netboot $load_bitstream_cmd -f $result_dir/runtime.bin -b $board_host
fi
