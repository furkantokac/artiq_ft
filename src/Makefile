TARGET := zc706
GWARGS := -V simple

all: ../build/firmware/armv7-none-eabihf/release/runtime ../build/runtime.bin

.PHONY: all


../build/pl.rs ../build/rustc-cfg: gateware/*
	mkdir -p ../build
	python gateware/$(TARGET).py -r ../build/pl.rs -c ../build/rustc-cfg $(GWARGS)

../build/firmware/armv7-none-eabihf/release/runtime: ../build/pl.rs ../build/rustc-cfg $(shell find . -print)
	cd runtime && \
	XBUILD_SYSROOT_PATH=`pwd`/../../build/sysroot \
	cargo xbuild --release \
		--target-dir ../../build/firmware \
		--no-default-features --features=target_$(TARGET)

../build/runtime.bin: ../build/firmware/armv7-none-eabihf/release/runtime
	llvm-objcopy -O binary ../build/firmware/armv7-none-eabihf/release/runtime ../build/runtime.bin
