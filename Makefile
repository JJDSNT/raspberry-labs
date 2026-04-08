TARGET_LE   := aarch64-unknown-none-softfloat
TARGET_BE   := aarch64_be-unknown-none-softfloat.json
TARGET_UEFI := aarch64-unknown-uefi

OUT_DIR   := out
KERNEL_LE := $(OUT_DIR)/kernel8.img
KERNEL_BE := $(OUT_DIR)/kernel8-be.img
EFI_OUT   := $(OUT_DIR)/BOOTAA64.EFI

AROS_MAIN := src/emu/c/omega2/memory/aros_main.h
AROS_EXT  := src/emu/c/omega2/memory/aros_ext.h
GEN_ROM   := python3 src/emu/rom/gen_rom.py

CARGO_NIGHTLY := cargo +nightly

.PHONY: \
	all \
	le be uefi \
	aros-rom \
	aros-sad-le aros-sad-be aros-sad \
	sdcard sdcard-be sdcard-uefi sdcard-uefi-be \
	clean

all: le

# ------------------------------------------------------------
# AROS ROM header generation
# ------------------------------------------------------------

aros-rom: _gen_aros

_gen_aros:
	@$(GEN_ROM)
	@echo "[ROM] AROS headers generated"

# ------------------------------------------------------------
# Standard kernel builds
# ------------------------------------------------------------

le:
	@mkdir -p $(OUT_DIR)
	cargo build --release --target $(TARGET_LE)
	cargo objcopy --release --target $(TARGET_LE) -- -O binary $(KERNEL_LE)
	@echo "[OK] $(KERNEL_LE) pronto"

be:
	@mkdir -p $(OUT_DIR)
	$(CARGO_NIGHTLY) build --release \
		--target $(TARGET_BE) \
		-Z build-std=core,compiler_builtins
	$(CARGO_NIGHTLY) objcopy --release \
		--target $(TARGET_BE) \
		-Z build-std=core,compiler_builtins \
		-- -O binary $(KERNEL_BE)
	@echo "[OK] $(KERNEL_BE) pronto"

uefi:
	@mkdir -p $(OUT_DIR)
	$(CARGO_NIGHTLY) build --release \
		--target $(TARGET_UEFI) \
		-Z build-std=core,compiler_builtins
	cp target/$(TARGET_UEFI)/release/raspi-labs.efi $(EFI_OUT)
	@echo "[OK] $(EFI_OUT) pronto"
	@echo "[>>] Copie para EFI/BOOT/BOOTAA64.EFI no cartão SD junto com RPI_EFI.fd"

# ------------------------------------------------------------
# AROS SAD builds
# Requires Cargo feature: aros-sad
# Ideal para compilar o subconjunto do rom/kernel via build.rs
# ------------------------------------------------------------

aros-sad: aros-sad-le

aros-sad-le: _gen_aros
	@mkdir -p $(OUT_DIR)
	cargo build --release --target $(TARGET_LE) --features aros-sad
	cargo objcopy --release --target $(TARGET_LE) --features aros-sad -- -O binary $(KERNEL_LE)
	@echo "[OK] $(KERNEL_LE) pronto com AROS SAD"

aros-sad-be: _gen_aros
	@mkdir -p $(OUT_DIR)
	$(CARGO_NIGHTLY) build --release \
		--target $(TARGET_BE) \
		-Z build-std=core,compiler_builtins \
		--features aros-sad
	$(CARGO_NIGHTLY) objcopy --release \
		--target $(TARGET_BE) \
		-Z build-std=core,compiler_builtins \
		--features aros-sad \
		-- -O binary $(KERNEL_BE)
	@echo "[OK] $(KERNEL_BE) pronto com AROS SAD"

# ------------------------------------------------------------
# SD card / runner helpers
# ------------------------------------------------------------

sdcard:
	./run.sh -s

sdcard-be:
	./run.sh -b -s

sdcard-uefi:
	./run.sh -u

sdcard-uefi-be:
	./run.sh -u -b

# ------------------------------------------------------------
# Cleanup
# ------------------------------------------------------------

clean:
	cargo clean
	rm -rf $(OUT_DIR)