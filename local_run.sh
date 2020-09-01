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

cd openocd
if [ $impure -eq 1 ]; then
    if [ $load_bitstream -eq 1 ]; then
        load_bitstream_cmd="-g ../build/gateware/top.bit"
    fi
    openocd -f zc706.cfg -c "load_image ../build/firmware/armv7-none-eabihf/debug/szl; resume 0; exit"
    sleep 5
    artiq_netboot $load_bitstream_cmd -f ../build/runtime.bin -b $board_host
else
    if [ $load_bitstream -eq 1 ]; then
        load_bitstream_cmd="-g ../result/top.bit"
    fi
    openocd -f zc706.cfg -c "load_image ../result/szl.elf; resume 0; exit"
    sleep 5
    artiq_netboot $load_bitstream_cmd -f ../result/runtime.bin -b $board_host
fi
