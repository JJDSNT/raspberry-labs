TARGET_LE = aarch64-unknown-none-softfloat
TARGET_BE = aarch64_be-unknown-none-softfloat.json

KERNEL_LE = kernel8.img
KERNEL_BE = kernel8-be.img

AROS_MAIN = src/emu/c/omega2/memory/aros_main.h
AROS_EXT  = src/emu/c/omega2/memory/aros_ext.h
GEN_ROM   = python3 src/emu/rom/gen_rom.py

.PHONY: le be aros clean

# Build with AROS ROM (generates headers from src/arosrom/ if needed)
# KS1.2 / KS1.3 are selected at boot time via the launcher TUI — no make target needed.
aros: _gen_aros le

_gen_aros:
	@$(GEN_ROM)
	@echo "[ROM] AROS headers generated"

le:
	cargo build --release --target $(TARGET_LE)
	cargo objcopy --release --target $(TARGET_LE) -- -O binary $(KERNEL_LE)
	@echo "[OK] $(KERNEL_LE) pronto"

be:
	cargo +nightly build --release \
		--target $(TARGET_BE) \
		-Z build-std=core,compiler_builtins
	cargo +nightly objcopy --release \
		--target $(TARGET_BE) \
		-Z build-std=core,compiler_builtins \
		-- -O binary $(KERNEL_BE)
	@echo "[OK] $(KERNEL_BE) pronto"

clean:
	cargo clean
	rm -f $(KERNEL_LE) $(KERNEL_BE)
