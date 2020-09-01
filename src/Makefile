VARIANT := simple

all: ../build/firmware/armv7-none-eabihf/release/szl ../build/firmware/armv7-none-eabihf/release/runtime

.PHONY: all


../build/pl.rs ../build/rustc-cfg: gateware/*
	mkdir -p ../build
	python gateware/zc706.py -r ../build/pl.rs -c ../build/rustc-cfg -V $(VARIANT)

../build/firmware/armv7-none-eabihf/release/runtime: ../build/pl.rs ../build/rustc-cfg $(shell find . -path ./szl -prune -o -print)
	XBUILD_SYSROOT_PATH=`pwd`/../build/sysroot cargo xbuild --release -p runtime --target-dir ../build/firmware
	llvm-objcopy -O binary ../build/firmware/armv7-none-eabihf/release/runtime ../build/runtime.bin

../build/firmware/armv7-none-eabihf/release/szl: .cargo/* armv7-none-eabihf.json Cargo.lock Cargo.toml szl/* szl/src/*
	XBUILD_SYSROOT_PATH=`pwd`/../build/sysroot cargo xbuild -p szl --target-dir ../build/firmware
