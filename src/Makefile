TARGET := zc706
GWARGS := -V nist_clock

all: ../build/runtime.bin

.PHONY: all

../build/pl.rs ../build/rustc-cfg ../build/mem.rs: gateware/* 
	mkdir -p ../build
	python gateware/$(TARGET).py -r ../build/pl.rs -c ../build/rustc-cfg -m ../build/mem.rs $(GWARGS)

../build/firmware/armv7-none-eabihf/release/runtime: ../build/pl.rs ../build/rustc-cfg ../build/mem.rs $(shell find . -print)
	cd runtime && \
	XBUILD_SYSROOT_PATH=`pwd`/../../build/sysroot \
	cargo xbuild --release \
		--target-dir ../../build/firmware \
		--no-default-features --features=target_$(TARGET)

../build/runtime.bin: ../build/firmware/armv7-none-eabihf/release/runtime
	llvm-objcopy -O binary ../build/firmware/armv7-none-eabihf/release/runtime ../build/runtime.bin

../build/firmware/armv7-none-eabihf/release/satman: ../build/pl.rs ../build/rustc-cfg ../build/mem.rs $(shell find . -print)
	cd satman && \
	XBUILD_SYSROOT_PATH=`pwd`/../../build/sysroot \
	cargo xbuild --release \
		--target-dir ../../build/firmware \
		--no-default-features --features=target_$(TARGET)

../build/satman.bin: ../build/firmware/armv7-none-eabihf/release/satman
	llvm-objcopy -O binary ../build/firmware/armv7-none-eabihf/release/satman ../build/satman.bin