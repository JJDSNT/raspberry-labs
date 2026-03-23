#!/bin/bash
set -e

DTB_DIR="./dtb"
DTB="$DTB_DIR/bcm2710-rpi-3-b-plus.dtb"
DTB_PATCHED="$DTB_DIR/bcm2710-rpi-3-b-plus-patched.dtb"
DTB_URL="https://github.com/dhruvvyas90/qemu-rpi-kernel/raw/master/native-emulation/dtbs/bcm2710-rpi-3-b-plus.dtb"
LAUNCHER_DIR="./launch"

TARGET_LE="aarch64-unknown-none-softfloat"
TARGET_BE="aarch64_be-unknown-none-softfloat.json"

CLEAN_LIGHT=0
CLEAN_FULL=0
BIG_ENDIAN=0
CREATE_SD=0

DISKS_DIR="./disks"
SDCARD_IMG="$(pwd)/sdcard.img"
FIRMWARE_DIR="./firmware"

# Firmware oficial RPi
FIRMWARE_BASE="https://github.com/raspberrypi/firmware/raw/master/boot"
FIRMWARE_FILES="bootcode.bin start.elf fixup.dat bcm2710-rpi-3-b-plus.dtb"

usage() {
    echo "Usage: $0 [-b] [-c] [-C] [-s]"
    echo "  -b    build big-endian (requer cargo nightly)"
    echo "  -c    limpeza leve (remove kernel gerado e artefatos temporários)"
    echo "  -C    limpeza total (cargo clean)"
    echo "  -s    cria/atualiza sdcard.img (serve para QEMU e SD card físico)"
    exit 1
}

while getopts "bcCsh" opt; do
    case $opt in
        b) BIG_ENDIAN=1 ;;
        c) CLEAN_LIGHT=1 ;;
        C) CLEAN_FULL=1 ;;
        s) CREATE_SD=1 ;;
        h) usage ;;
        *) usage ;;
    esac
done

if [ "$BIG_ENDIAN" -eq 1 ]; then
    KERNEL="kernel8-be.img"
    TARGET="$TARGET_BE"
    CARGO_TOOLCHAIN="+nightly"
    CARGO_EXTRA="-Z build-std=core,compiler_builtins"
    echo "[INFO] Build: big-endian (requer nightly)"
else
    KERNEL="kernel8.img"
    TARGET="$TARGET_LE"
    CARGO_TOOLCHAIN=""
    CARGO_EXTRA=""
    echo "[INFO] Build: little-endian (padrão)"
fi

# ---------------------------------------------------------------------------
# SD card unificado (sdcard.img — FAT32 raw, serve para QEMU e hardware RPi)
# ---------------------------------------------------------------------------

download_firmware() {
    mkdir -p "$FIRMWARE_DIR"
    local DOWNLOADER=""
    if command -v curl >/dev/null 2>&1; then
        DOWNLOADER="curl -L -o"
    elif command -v wget >/dev/null 2>&1; then
        DOWNLOADER="wget -O"
    else
        echo "[ERROR] curl ou wget necessário para baixar firmware"
        exit 1
    fi

    for f in $FIRMWARE_FILES; do
        if [ ! -f "$FIRMWARE_DIR/$f" ]; then
            echo "[FW] Baixando $f..."
            $DOWNLOADER "$FIRMWARE_DIR/$f" "$FIRMWARE_BASE/$f"
        fi
    done
    echo "[FW] Firmware OK"
}

create_sdcard_image() {
    command -v mcopy >/dev/null 2>&1 || {
        echo "[ERROR] mtools não encontrado"
        echo "[HINT]  sudo apt install mtools"
        exit 1
    }

    download_firmware

    local SIZE_MB=128
    echo "[SD] Criando $SDCARD_IMG (${SIZE_MB}MB, FAT32)..."
    dd if=/dev/zero of="$SDCARD_IMG" bs=1M count="$SIZE_MB" status=none
    mformat -i "$SDCARD_IMG" -F -v "RASPI" ::

    # Firmware RPi (também ignorado pelo QEMU, mas não faz mal)
    for f in $FIRMWARE_FILES; do
        if [ -f "$FIRMWARE_DIR/$f" ]; then
            echo "[SD] + $f"
            mcopy -i "$SDCARD_IMG" "$FIRMWARE_DIR/$f" ::
        fi
    done

    # Kernel
    echo "[SD] + kernel8.img ($(du -h "$KERNEL" | cut -f1))"
    mcopy -i "$SDCARD_IMG" "$KERNEL" "::kernel8.img"

    # config.txt
    cat > /tmp/rpi_config.txt << 'EOF'
arm_64bit=1
kernel=kernel8.img
enable_uart=1
init_uart_clock=48000000
EOF
    mcopy -i "$SDCARD_IMG" /tmp/rpi_config.txt "::config.txt"

    # cmdline.txt
    local CMDLINE="demo=flame"
    if [ -f "$DISKS_DIR/disk0.adf" ]; then
        CMDLINE="demo=omega df0=disk0.adf df1=disk1.adf"
        for ROM_FILE in "$DISKS_DIR"/*.rom "$DISKS_DIR"/*.ROM; do
            [ -f "$ROM_FILE" ] || continue
            ROM_NAME="$(basename "$ROM_FILE")"
            CMDLINE="$CMDLINE rom=$ROM_NAME"
            echo "[SD] ROM: $ROM_NAME"
            mcopy -i "$SDCARD_IMG" "$ROM_FILE" "::$ROM_NAME"
            break
        done
    fi
    printf '%s' "$CMDLINE" > /tmp/rpi_cmdline.txt
    mcopy -i "$SDCARD_IMG" /tmp/rpi_cmdline.txt "::cmdline.txt"
    echo "[SD] cmdline: $CMDLINE"

    # ADFs
    local ADDED=0
    for n in 0 1; do
        local ADF="$DISKS_DIR/disk${n}.adf"
        if [ -f "$ADF" ]; then
            echo "[SD] + disk${n}.adf ($(du -h "$ADF" | cut -f1))"
            mcopy -i "$SDCARD_IMG" "$ADF" "::disk${n}.adf"
            ADDED=$((ADDED + 1))
        fi
    done

    echo ""
    echo "[SD] sdcard.img pronto! ($ADDED disco(s))"
    echo "     QEMU:     ./run.sh  (usa sdcard.img automaticamente)"
    echo "     Hardware: sudo dd if=sdcard.img of=/dev/sdX bs=4M status=progress && sync"
    echo ""
}

if [ "$CREATE_SD" -eq 1 ]; then
    create_sdcard_image
fi

# ---------------------------------------------------------------------------
# Pré-requisitos
# ---------------------------------------------------------------------------

command -v cargo  >/dev/null 2>&1 || { echo "[ERROR] cargo não encontrado"; exit 1; }
command -v go >/dev/null 2>&1 || { echo "[ERROR] go não encontrado"; exit 1; }
command -v fdtput >/dev/null 2>&1 || {
    echo "[ERROR] fdtput não encontrado"
    echo "[HINT]  sudo apt install device-tree-compiler"
    exit 1
}

cargo objcopy --version >/dev/null 2>&1 || {
    echo "[ERROR] cargo objcopy não disponível"
    echo "[HINT]  cargo install cargo-binutils && rustup component add llvm-tools-preview"
    exit 1
}

if [ "$BIG_ENDIAN" -eq 1 ]; then
    cargo +nightly --version >/dev/null 2>&1 || {
        echo "[ERROR] cargo nightly não encontrado"
        echo "[HINT]  rustup toolchain install nightly"
        exit 1
    }
fi

# ---------------------------------------------------------------------------
# DTB base (só necessário para QEMU)
# ---------------------------------------------------------------------------

mkdir -p "$DTB_DIR"
if [ ! -f "$DTB" ]; then
        echo "[DTB] Baixando para $DTB..."
        if command -v curl >/dev/null 2>&1; then
            curl -L -o "$DTB" "$DTB_URL"
        elif command -v wget >/dev/null 2>&1; then
            wget -O "$DTB" "$DTB_URL"
        else
            echo "[ERROR] curl ou wget necessário para baixar o DTB"
            exit 1
        fi
        echo "[DTB] OK — $DTB pronto"
fi

# ---------------------------------------------------------------------------
# Limpeza
# ---------------------------------------------------------------------------

if [ "$CLEAN_LIGHT" -eq 1 ]; then
    echo "[CLEAN] Limpeza leve..."
    rm -f "$KERNEL"
    rm -f "$DTB_PATCHED"
fi

if [ "$CLEAN_FULL" -eq 1 ]; then
    echo "[CLEAN] Limpeza total..."
    cargo clean
    rm -f kernel8.img kernel8-be.img
    rm -f "$DTB_PATCHED"
fi

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------

echo "[BUILD] Compilando kernel..."
# shellcheck disable=SC2086
cargo $CARGO_TOOLCHAIN build --release --target "$TARGET" $CARGO_EXTRA

echo "[BUILD] Gerando $KERNEL..."
# shellcheck disable=SC2086
cargo $CARGO_TOOLCHAIN objcopy --release --target "$TARGET" $CARGO_EXTRA -- -O binary "$KERNEL"

if [ ! -f "$KERNEL" ]; then
    echo "[ERROR] $KERNEL não foi gerado"
    exit 1
fi

echo "[BUILD] OK — $KERNEL pronto"

# ---------------------------------------------------------------------------
# Launcher TUI (QEMU)
# ---------------------------------------------------------------------------

if [ ! -d "$LAUNCHER_DIR" ]; then
    echo "[ERROR] Diretório $LAUNCHER_DIR não encontrado"
    exit 1
fi

if [ ! -f "$LAUNCHER_DIR/go.sum" ]; then
    echo "[LAUNCH] Primeira execução — baixando dependências Go..."
    (cd "$LAUNCHER_DIR" && go mod tidy) || {
        echo "[ERROR] go mod tidy falhou"
        exit 1
    }
fi

echo "[LAUNCH] Iniciando seletor de demos..."
(
    cd "$LAUNCHER_DIR"
    KERNEL_PATH="$(cd .. && pwd)/$KERNEL" \
    DTB_PATH="$(cd .. && pwd)/$DTB" \
    DTB_DIR="$(cd .. && pwd)/$DTB_DIR" \
    SD_IMG_PATH="$SDCARD_IMG" \
    go run .
)
