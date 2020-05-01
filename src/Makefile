all: target/armv7-none-eabihf/release/szl

clean:
	rm -f runtime/src/pl.rs
	rm -f szl/src/payload.bin
	rm -f szl/src/payload.bin.lzma
	rm -rf target

.PHONY: all clean


runtime/src/pl.rs: zc706.py
	python zc706.py -r runtime/src/pl.rs

target/armv7-none-eabihf/release/runtime: .cargo/* armv7-none-eabihf.json Cargo.lock Cargo.toml libdyld/* libdyld/src/* runtime/* runtime/src/* runtime/src/pl.rs
	cargo xbuild --release -p runtime

szl/src/payload.bin.lzma: target/armv7-none-eabihf/release/runtime
	llvm-objcopy -O binary target/armv7-none-eabihf/release/runtime szl/src/payload.bin
	lzma --keep -f szl/src/payload.bin

target/armv7-none-eabihf/release/szl: .cargo/* armv7-none-eabihf.json Cargo.lock Cargo.toml szl/* szl/src/* szl/src/payload.bin.lzma
	cargo xbuild --release -p szl
