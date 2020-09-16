#!/bin/bash

set -e
cd "$(dirname "$0")"

cp ../boards/layout_hifive1.ld ../layout.ld
KERNEL_HEAP_SIZE=1024 PLATFORM=hifive1 cargo rrv32imac --example console
echo "TO EXIT: Ctrl-A x"
../tock/tools/qemu-build/riscv32-softmmu/qemu-system-riscv32 \
	-M sifive_e,revb=true \
	-kernel ../tock/target/riscv32imac-unknown-none-elf/release/hifive1 \
	-device loader,file=../target/riscv32imac-unknown-none-elf/tab/hifive1/console/rv32imac.tbf,addr=0x20040000 \
	-nographic
