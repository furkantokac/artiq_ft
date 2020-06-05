#!/usr/bin/env bash

set -e

impure=0

while getopts "h:i" opt; do
    case "$opt" in
    \?) exit 0
        ;;
    i)  impure=1
        ;;
    esac
done

cd openocd
if [ $impure -eq 1 ]; then
    openocd -f zc706.cfg -c 'pld load 0 ../build/gateware/top.bit; load_image ../build/firmware/armv7-none-eabihf/release/szl; resume 0; exit'
else
    openocd -f zc706.cfg -c 'pld load 0 ../result/top.bit; load_image ../result/szl.elf; resume 0; exit'
fi
