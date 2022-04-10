.PHONY: all debug release clean run-debug run-release dump-debug dump-release

COMMON_FLAGS = -Zbuild-std=core,alloc

all: debug

debug:
	cargo build $(COMMON_FLAGS)
	cp target/thumbv7em-none-eabi/debug/demodemo image-debug.elf

release:
	cargo build $(COMMON_FLAGS) --release
	cp target/thumbv7em-none-eabi/release/demodemo image-release.elf

run-debug: debug
	qemu-system-gnuarmeclipse -cpu cortex-m4 -machine STM32F4-Discovery -nographic -kernel image-debug.elf

run-release: release
	qemu-system-gnuarmeclipse -cpu cortex-m4 -machine STM32F4-Discovery -nographic -kernel image-release.elf

gdb-debug: debug
	qemu-system-gnuarmeclipse -cpu cortex-m4 -machine STM32F4-Discovery -nographic -S -gdb tcp::3333 -kernel image-debug.elf

gdb-release: release
	qemu-system-gnuarmeclipse -cpu cortex-m4 -machine STM32F4-Discovery -nographic -S -gdb tcp::3333 -kernel image-release.elf

dump-debug: debug
	arm-none-eabi-objdump -d image-debug.elf > dump-debug.asm

dump-release: release
	arm-none-eabi-objdump -d image-release.elf > dump-release.asm

clean:
	cargo clean
	rm -f image-*.elf dump-*.asm
