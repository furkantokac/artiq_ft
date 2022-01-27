#!/usr/bin/env bash

set -e

if [ -z "$OPENOCD_ZYNQ" ]; then
    echo "OPENOCD_ZYNQ environment variable must be set"
    exit 1
fi
if [ -z "$SZL" ]; then
    echo "SZL environment variable must be set"
    exit 1
fi

impure=0
load_bitstream=1
board_type="kasli_soc"
fw_type="runtime"

while getopts "ilb:t:f:" opt; do
    case "$opt" in
    \?) exit 1
        ;;
    i)  impure=1
        ;;
    l)  load_bitstream=0
        ;;
    b)  board_host=$OPTARG
        ;;
    t)  board_type=$OPTARG
        ;;
    f)  fw_type=$OPTARG
        ;;
    esac
done

if [ -z "$board_host" ]; then
    case $board_type in
    kasli_soc) board_host="192.168.1.56";;
    zc706) board_host="192.168.1.52";;
    *) echo "Unknown board type"; exit 1;;
    esac
fi

load_bitstream_cmd=""

build_dir=`pwd`/build
result_dir=`pwd`/result
cd $OPENOCD_ZYNQ
openocd -f $board_type.cfg -c "load_image $SZL/szl-$board_type.elf; resume 0; exit"
sleep 5
if [ $impure -eq 1 ]; then
    if [ $load_bitstream -eq 1 ]; then
        load_bitstream_cmd="-g $build_dir/gateware/top.bit"
    fi
    artiq_netboot $load_bitstream_cmd -f $build_dir/$fw_type.bin -b $board_host
else
    if [ $load_bitstream -eq 1 ]; then
        load_bitstream_cmd="-g $result_dir/top.bit"
    fi
    artiq_netboot $load_bitstream_cmd -f $result_dir/$fw_type.bin -b $board_host
fi