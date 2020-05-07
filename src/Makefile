all: ../build/firmware/armv7-none-eabihf/release/szl

.PHONY: all


../build/pl.rs: zc706.py
	mkdir -p ../build
	python zc706.py -r ../build/pl.rs

../build/firmware/armv7-none-eabihf/release/runtime: .cargo/* armv7-none-eabihf.json Cargo.lock Cargo.toml libdyld/* libdyld/src/* runtime/* runtime/src/* ../build/pl.rs
	cargo xbuild --release -p runtime --target-dir ../build/firmware

../build/szl-payload.bin.lzma: ../build/firmware/armv7-none-eabihf/release/runtime
	llvm-objcopy -O binary ../build/firmware/armv7-none-eabihf/release/runtime ../build/szl-payload.bin
	lzma --keep -f ../build/szl-payload.bin

../build/firmware/armv7-none-eabihf/release/szl: .cargo/* armv7-none-eabihf.json Cargo.lock Cargo.toml szl/* szl/src/* ../build/szl-payload.bin.lzma
	cargo xbuild --release -p szl --target-dir ../build/firmware
