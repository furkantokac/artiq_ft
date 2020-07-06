#!/usr/bin/env bash

set -e

impure=0
load_bitstream=1

while getopts "h:il" opt; do
    case "$opt" in
    \?) exit 1
        ;;
    i)  impure=1
        ;;
    l)  load_bitstream=0
        ;;
    esac
done

load_bitstream_cmd=""

cd openocd
if [ $impure -eq 1 ]; then
    if [ $load_bitstream -eq 1 ]; then
        load_bitstream_cmd="pld load 0 ../build/gateware/top.bit;"
    fi
    openocd -f zc706.cfg -c "$load_bitstream_cmd load_image ../build/firmware/armv7-none-eabihf/release/szl; resume 0; exit"
else
    if [ $load_bitstream -eq 1 ]; then
        load_bitstream_cmd="pld load 0 ../result/top.bit;"
    fi
    openocd -f zc706.cfg -c "$load_bitstream_cmd load_image ../result/szl.elf; resume 0; exit"
fi
