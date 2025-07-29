NAME := blog_v1
BUILD_DIR := build
ASM_DIR := asm
ASM_SRCS := $(wildcard $(ASM_DIR)/*.asm)
ASM_OBJS := $(patsubst $(ASM_DIR)/%.asm, $(BUILD_DIR)/$(ASM_DIR)/%.o, $(ASM_SRCS))
ISOFILES := $(BUILD_DIR)/isofiles
KERNEL := $(BUILD_DIR)/kernel.bin
ISO := $(BUILD_DIR)/os.iso
TARGET := $(ASM_DIR)/$(NAME)
RUST_OS := target/$(NAME)/debug/lib$(NAME).a
LINKER_SCRIPT := linker.ld
GRUB_CFG := grub.cfg

all: $(ISO)

re: clean all

run: all
	@qemu-system-x86_64 -cdrom $(ISO)

rerun: clean run

clean:
	cargo clean

$(ISO): $(KERNEL) $(GRUB_CFG) $(ASM_SRCS) $(NAME).json
	@mkdir -p $(ISOFILES)/boot/grub
	@cp $(KERNEL) $(ISOFILES)/boot/kernel.bin
	@cp $(GRUB_CFG) $(ISOFILES)/boot/grub
	@grub-mkrescue -o $(ISO) $(GRUB_FLAGS) $(ISOFILES)
	@rm -rf $(ISOFILES)

$(KERNEL): $(RUST_OS) $(ASM_OBJS) $(LINKER_SCRIPT)
	@ld $(LD_FLAGS) -n --gc-sections -T $(LINKER_SCRIPT) -o $(KERNEL) $(ASM_OBJS) $(RUST_OS)

$(RUST_OS):
	@export RUST_TARGET_PATH=$(shell pwd) ; cargo build --target $(NAME) $(CARGO_FLAGS)

$(ASM_OBJS): $(BUILD_DIR)/$(ASM_DIR)/%.o: $(ASM_DIR)/%.asm
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