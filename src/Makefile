VARIANT := simple

all: ../build/firmware/armv7-none-eabihf/release/szl

.PHONY: all


../build/pl.rs: gateware/zc706.py
	mkdir -p ../build
	python gateware/zc706.py -r ../build/pl.rs -V $(VARIANT)

../build/firmware/armv7-none-eabihf/release/runtime: ../build/pl.rs $(shell find . -path ./szl -prune -o -print)
	XBUILD_SYSROOT_PATH=`pwd`/../build/sysroot cargo xbuild --release -p runtime --target-dir ../build/firmware

../build/szl-payload.bin.lzma: ../build/firmware/armv7-none-eabihf/release/runtime
	llvm-objcopy -O binary ../build/firmware/armv7-none-eabihf/release/runtime ../build/szl-payload.bin
	lzma --keep -f ../build/szl-payload.bin

../build/firmware/armv7-none-eabihf/release/szl: .cargo/* armv7-none-eabihf.json Cargo.lock Cargo.toml szl/* szl/src/* ../build/szl-payload.bin.lzma
	XBUILD_SYSROOT_PATH=`pwd`/../build/sysroot cargo xbuild --release -p szl --target-dir ../build/firmware
