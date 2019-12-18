
arch ?= x86_64
kernel := build/kernel.bin
iso := build/redleaf.iso

linker_script := linker.ld
grub_cfg := boot/grub.cfg
#assembly_source_files := $(wildcard src/*.asm)
#assembly_object_files := $(patsubst src/%.asm, build/%.o, $(assembly_source_files))

target ?= $(arch)-redleaf
rust_os := target/$(target)/debug/libredleaf.a
xv6fs = usr/mkfs/build/fs.img

qemu_common := -m 128m -vga std -s
qemu_common := $(qemu_common) -cdrom $(iso)
qemu_common := $(qemu_common) -no-reboot -no-shutdown -d int,cpu_reset
qemu_common := $(qemu_common) -drive file=disk.img,index=0,media=$(xv6fs),format=raw
qemu_common := $(qemu_common) -smp 2

# https://superuser.com/a/1412150
qemu_nox := -nographic -chardev stdio,id=char0,mux=on,logfile=serial.log,signal=off -serial chardev:char0 -mon chardev=char0

qemu_x := -serial file:serial.log

.PHONY: all
all: $(kernel)

.PHONY: release
release: $(releaseKernel)

.PHONY: clean
clean:
	make -C sys/init clean
	rm -rf build
	cargo clean
	make -C usr/mkfs clean

.PHONY: run
run: qemu

.PHONY: run-nox
run-nox: qemu-nox

.PHONY: qemu
qemu: $(iso) disk.img
	qemu-system-x86_64 $(qemu_common) $(qemu_x)

.PHONY: qemu-gdb
qemu-gdb: $(iso) disk.img
	qemu-system-x86_64 $(qemu_common) $(qemu_x) -S

.PHONY: qemu-gdb-nox
qemu-gdb-nox: $(iso) disk.img
	qemu-system-x86_64 $(qemu_common) $(qemu_nox) -S

.PHONY: qemu-nox
qemu-nox: $(iso) disk.img
	qemu-system-x86_64 $(qemu_common) $(qemu_nox)

.PHONY: qemu-nox-cloudlab
qemu-nox-cloudlab: $(iso)
	$(eval pciflag := $(shell sudo ./rebind-82599es.sh))
	sudo qemu-system-x86_64 $(qemu_common) $(qemu_nox) $(pciflag)

.PHONY: qemu-efi-nox
qemu-efi-nox: $(iso) disk.img ovmf-code
	qemu-system-x86_64 $(qemu_common) $(qemu_nox) -bios OVMF_CODE.fd

disk.img:
	make -C usr/mkfs 
	#fallocate -l 512M disk.img

ovmf-code:
	echo "Getting OVMF_CODE.fd is not implemented..."

.PHONY: iso
iso: $(iso)
	@echo "Done"

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	cp $(kernel) build/isofiles/boot/kernel.bin
	cp $(grub_cfg) build/isofiles/boot/grub
	grub-mkrescue -o $(iso) build/isofiles #2> /dev/null
	@rm -r build/isofiles

$(kernel): kernel $(rust_os) bootblock entryother entry $(linker_script) init
	ld -n --gc-sections -T $(linker_script) -o $(kernel) build/entry.o build/boot.o build/multiboot_header.o $(rust_os) -b binary build/entryother.bin sys/init/build/init

.PHONY: init
init:
	make -C sys/init

.PHONY: kernel
kernel:
	@RUST_TARGET_PATH=$(32shell pwd) cargo xbuild --target x86_64-redleaf.json

# compile assembly files for the exception entry code
.PHONY: entry
entry: src/arch/entry_64.S 
	@mkdir -p build
	gcc -fno-builtin -fno-strict-aliasing -Wall -MD -ggdb -fno-pic -nostdinc -I. -o build/entry.o -c src/arch/entry_64.S


# compile assembly files
.PHONY: bootblock
bootblock: src/boot.asm src/multiboot_header.asm
	@mkdir -p build
	nasm -felf64 src/boot.asm -o build/boot.o
	nasm -felf64 src/multiboot_header.asm -o build/multiboot_header.o

# compile assembly files
.PHONY: entryother
entryother: src/entryother.asm
	@mkdir -p build
	nasm -felf64 src/entryother.asm -o build/entryother.o
	ld -N -e start_others16 -Ttext 0x7000 -o build/entryother.out build/entryother.o
	objcopy -S -O binary -j .text build/entryother.out build/entryother.bin

