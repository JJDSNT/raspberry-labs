TARGET_LE = aarch64-unknown-none-softfloat
TARGET_BE = aarch64_be-unknown-none-softfloat.json

KERNEL_LE = kernel8.img
KERNEL_BE = kernel8-be.img

.PHONY: le be clean

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
