DEBUG ?= true

ifeq ($(DEBUG),true)
BUILD_MODE := debug
else ifeq ($(DEBUG),false)
BUILD_MODE := release
CARGO_FLAGS := --release
GRUB_FLAGS := --compress xz
else
$(error DEBUG must be either true or false)
endif

ARCH := x86_64
BUILD := build/$(ARCH)/$(BUILD_MODE)
ISOFILES := $(BUILD)/isofiles
KERNEL := $(BUILD)/kernel-$(ARCH).bin
ISO := $(BUILD)/os-$(ARCH).iso
TARGET := arch/$(ARCH)/blog_v1
RUST_OS := target/$(TARGET)/$(BUILD_MODE)/libblog_v1.a
LINKER_SCRIPT := arch/linker.ld
GRUB_CFG := arch/grub.cfg
ASM_SRCS := $(wildcard arch/$(ARCH)/*.asm)
ASM_OBJS := $(patsubst arch/$(ARCH)/%.asm, $(BUILD)/arch/$(ARCH)/%.o, $(ASM_SRCS))
QEMU := qemu-system-$(ARCH)

all: $(ISO)

re: clean all

run: all
	@$(QEMU) -cdrom $(ISO)

rerun: clean run

clean:
	cargo clean || true
	rm -rf build rust_analyzer target || true
	mv build rust_analyzer target $$HOME/.local/share/Trash/files/ || true

$(ISO): $(KERNEL) $(GRUB_CFG) $(ASM_SRCS) $(TARGET).json
	@mkdir -p $(ISOFILES)/boot/grub
	@cp $(KERNEL) $(ISOFILES)/boot/kernel.bin
	@cp $(GRUB_CFG) $(ISOFILES)/boot/grub
	@grub-mkrescue -o $(ISO) $(GRUB_FLAGS) $(ISOFILES)
	@rm -rf $(ISOFILES)

$(KERNEL): $(RUST_OS) $(ASM_OBJS) $(LINKER_SCRIPT)
	@ld $(LD_FLAGS) -n --gc-sections -T $(LINKER_SCRIPT) -o $(KERNEL) $(ASM_OBJS) $(RUST_OS)

$(RUST_OS):
	@export RUST_TARGET_PATH=$(shell pwd) ; cargo build --target $(TARGET) $(CARGO_FLAGS)

$(ASM_OBJS): $(BUILD)/arch/$(ARCH)/%.o: arch/$(ARCH)/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -f elf64 $< -o $@

define compile_from_source
    @rm -rf source_dir source.tar.gz
	@wget -O source.tar.gz $(1)
    @mkdir source_dir && tar xvf source.tar.gz -C source_dir --strip-components=1
    @cd source_dir && ./configure --prefix=$$HOME/.local && make -j && make install
    @rm -rf source_dir source.tar.gz
endef

install_requirements: uninstall_requirements
	$(call compile_from_source,ftp://ftp.gnu.org/gnu/grub/grub-2.06.tar.xz)
	$(call compile_from_source,https://www.gnu.org/software/xorriso/xorriso-1.5.4.tar.gz)

uninstall_requirements:
	@rm -rf source_dir source.tar.gz
	@rm -rf $$HOME/.local/bin/grub-*
	@rm -rf $$HOME/.local/bin/xorriso*
	@rm -rf $$HOME/.local/bin/osirrox
	@rm -rf $$HOME/.local/bin/xorrecord
	@rm -rf $$HOME/.local/etc/grub.d
	@rm -rf $$HOME/.local/share/grub

.PHONY: all re run rerun clean $(RUST_OS) install_requirements uninstall_requirements