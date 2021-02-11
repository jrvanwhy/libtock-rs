#!/bin/bash

set -e

cd "$(dirname "$0")"/..
APP_HEAP_SIZE=256 KERNEL_HEAP_SIZE=256 LIBTOCK_PLATFORM=hifive1 PLATFORM=hifive1 \
	cargo run --release -p libtock2 --example empty_main --target=riscv32imac-unknown-none-elf
tock/tools/qemu-build/riscv32-softmmu/qemu-system-riscv32 \
	-M sifive_e,revb=true \
	-kernel tock2/target/riscv32imac-unknown-none-elf/release/hifive1 \
	-device loader,file=target/riscv32imac-unknown-none-elf/tab/hifive1/empty_main/rv32imac.tbf,addr=0x20040000 \
	-nographic
