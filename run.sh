#!/bin/bash
set -e

DTB_DIR="./dtb_qemu"
DTB="$DTB_DIR/bcm2710-rpi-3-b-plus.dtb"
DTB_PATCHED="$DTB_DIR/bcm2710-rpi-3-b-plus-patched.dtb"
DTB_URL="https://github.com/dhruvvyas90/qemu-rpi-kernel/raw/master/native-emulation/dtbs/bcm2710-rpi-3-b-plus.dtb"
LAUNCHER_DIR="./launch"

TARGET_LE="aarch64-unknown-none-softfloat"
TARGET_BE="aarch64_be-unknown-none-softfloat.json"
TARGET_UEFI="aarch64-unknown-uefi"

CLEAN_LIGHT=0
CLEAN_FULL=0
BIG_ENDIAN=0
CREATE_SD=0
UEFI_MODE=0

DISKS_DIR="./disks"
OUT_DIR="./out"
EFI_OUT="$OUT_DIR/BOOTAA64.EFI"
SDCARD_IMG="$(pwd)/out/sdcard.img"
FIRMWARE_DIR="./firmware"
SDCARD_OVERRIDES="./sdcard"

# Firmware oficial RPi
FIRMWARE_BASE="https://github.com/raspberrypi/firmware/raw/master/boot"
FIRMWARE_FILES="bootcode.bin start.elf fixup.dat bcm2710-rpi-3-b-plus.dtb"

usage() {
    echo "Usage: $0 [-b] [-c] [-C] [-s] [-u]"
    echo "  -b    build big-endian (requer cargo nightly)"
    echo "  -c    limpeza leve (remove kernel gerado e artefatos temporários)"
    echo "  -C    limpeza total (cargo clean)"
    echo "  -s    cria/atualiza sdcard.img bare-metal (QEMU e SD card físico)"
    echo "  -u    cria/atualiza sdcard.img UEFI — pftf/RPi3 (implica -s, sem QEMU)"
    echo ""
    echo "Overrides em $SDCARD_OVERRIDES/:"
    echo "  cmdline.txt  — kernel cmdline bare-metal (padrão: demo=flame)"
    echo "  config.txt   — boot config (padrão gerado automaticamente)"
    echo "  RPI_EFI.fd   — firmware UEFI pftf (pode também estar em firmware/)"
    exit 1
}

while getopts "bcCsuh" opt; do
    case $opt in
        b) BIG_ENDIAN=1 ;;
        c) CLEAN_LIGHT=1 ;;
        C) CLEAN_FULL=1 ;;
        s) CREATE_SD=1 ;;
        u) UEFI_MODE=1 ; CREATE_SD=1 ;;
        h) usage ;;
        *) usage ;;
    esac
done

if [ "$BIG_ENDIAN" -eq 1 ]; then
    KERNEL="$OUT_DIR/kernel8-be.img"
    TARGET="$TARGET_BE"
    CARGO_TOOLCHAIN="+nightly"
    CARGO_EXTRA="-Z build-std=core,compiler_builtins"
    echo "[INFO] Build: big-endian (requer nightly)"
else
    KERNEL="$OUT_DIR/kernel8.img"
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

    command -v mformat >/dev/null 2>&1 || {
        echo "[ERROR] mformat não encontrado"
        echo "[HINT]  sudo apt install mtools"
        exit 1
    }

    download_firmware

    local SIZE_MB=128
    echo "[SD] Criando $SDCARD_IMG (bare-metal, ${SIZE_MB}MB, FAT32)..."
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
    echo "[SD] + $KERNEL ($(du -h "$KERNEL" | cut -f1))"
    mcopy -i "$SDCARD_IMG" "$KERNEL" "::$(basename "$KERNEL")"

    # config.txt — usa sdcard/config.txt se existir, senão gera default
    if [ -f "$SDCARD_OVERRIDES/config.txt" ]; then
        echo "[SD] config.txt: usando $SDCARD_OVERRIDES/config.txt"
        mcopy -i "$SDCARD_IMG" "$SDCARD_OVERRIDES/config.txt" "::config.txt"
    else
        cat > /tmp/rpi_config.txt << EOF
arm_64bit=1
kernel=$(basename "$KERNEL")
enable_uart=1
init_uart_clock=48000000
EOF
        mcopy -i "$SDCARD_IMG" /tmp/rpi_config.txt "::config.txt"
        echo "[SD] config.txt: gerado (uart=1, kernel=$(basename "$KERNEL"))"
    fi

    # cmdline.txt — usa sdcard/cmdline.txt se existir, senão usa default
    if [ -f "$SDCARD_OVERRIDES/cmdline.txt" ]; then
        echo "[SD] cmdline.txt: usando $SDCARD_OVERRIDES/cmdline.txt"
        echo "     -> $(cat "$SDCARD_OVERRIDES/cmdline.txt")"
        mcopy -i "$SDCARD_IMG" "$SDCARD_OVERRIDES/cmdline.txt" "::cmdline.txt"
    else
        local CMDLINE="demo=flame"
        printf '%s' "$CMDLINE" > /tmp/rpi_cmdline.txt
        mcopy -i "$SDCARD_IMG" /tmp/rpi_cmdline.txt "::cmdline.txt"
        echo "[SD] cmdline.txt: $CMDLINE (padrão)"
    fi

    # Copia todo o conteúdo de ./disks para a raiz da imagem
    local COPIED=0
    if [ -d "$DISKS_DIR" ]; then
        shopt -s nullglob dotglob
        local DISK_ITEMS=("$DISKS_DIR"/*)
        shopt -u dotglob

        if [ ${#DISK_ITEMS[@]} -gt 0 ]; then
            echo "[SD] Copiando conteúdo de $DISKS_DIR..."
            mcopy -i "$SDCARD_IMG" -s "${DISK_ITEMS[@]}" ::
            COPIED=${#DISK_ITEMS[@]}
        else
            echo "[SD] $DISKS_DIR está vazio"
        fi
        shopt -u nullglob
    else
        echo "[SD] Diretório $DISKS_DIR não existe"
    fi

    echo ""
    echo "[SD] sdcard.img bare-metal pronto! ($COPIED item(ns) copiado(s) de disks/)"
    echo "     QEMU:     ./run.sh  (usa sdcard.img automaticamente)"
    echo "     Hardware: sudo dd if=sdcard.img of=/dev/sdX bs=4M status=progress && sync"
    echo ""
}

create_uefi_sdcard_image() {
    command -v mcopy >/dev/null 2>&1 || {
        echo "[ERROR] mtools não encontrado"
        echo "[HINT]  sudo apt install mtools"
        exit 1
    }
    command -v mformat >/dev/null 2>&1 || {
        echo "[ERROR] mformat não encontrado"
        echo "[HINT]  sudo apt install mtools"
        exit 1
    }
    command -v mmd >/dev/null 2>&1 || {
        echo "[ERROR] mmd não encontrado"
        echo "[HINT]  sudo apt install mtools"
        exit 1
    }

    download_firmware

    local SIZE_MB=128
    echo "[SD] Criando $SDCARD_IMG (UEFI, ${SIZE_MB}MB, FAT32)..."
    dd if=/dev/zero of="$SDCARD_IMG" bs=1M count="$SIZE_MB" status=none
    mformat -i "$SDCARD_IMG" -F -v "RASPI" ::

    # Firmware RPi base (necessário para pftf UEFI inicializar)
    for f in $FIRMWARE_FILES; do
        if [ -f "$FIRMWARE_DIR/$f" ]; then
            echo "[SD] + $f"
            mcopy -i "$SDCARD_IMG" "$FIRMWARE_DIR/$f" ::
        fi
    done

    # RPI_EFI.fd — firmware pftf/RPi3 UEFI
    local EFI_FW=""
    if [ -f "$FIRMWARE_DIR/RPI_EFI.fd" ]; then
        EFI_FW="$FIRMWARE_DIR/RPI_EFI.fd"
    elif [ -f "$SDCARD_OVERRIDES/RPI_EFI.fd" ]; then
        EFI_FW="$SDCARD_OVERRIDES/RPI_EFI.fd"
    fi

    if [ -n "$EFI_FW" ]; then
        echo "[SD] + RPI_EFI.fd"
        mcopy -i "$SDCARD_IMG" "$EFI_FW" "::RPI_EFI.fd"
    else
        echo "[WARN] RPI_EFI.fd não encontrado — SD card UEFI ficará incompleto"
        echo "       Baixe de: https://github.com/pftf/RPi3/releases"
        echo "       Coloque em: $FIRMWARE_DIR/RPI_EFI.fd  ou  $SDCARD_OVERRIDES/RPI_EFI.fd"
    fi

    # EFI/BOOT/BOOTAA64.EFI
    if [ ! -f "$EFI_OUT" ]; then
        echo "[ERROR] $EFI_OUT não encontrado — execute: make uefi"
        exit 1
    fi
    mmd -i "$SDCARD_IMG" "::EFI"
    mmd -i "$SDCARD_IMG" "::EFI/BOOT"
    echo "[SD] + EFI/BOOT/BOOTAA64.EFI ($(du -h "$EFI_OUT" | cut -f1))"
    mcopy -i "$SDCARD_IMG" "$EFI_OUT" "::EFI/BOOT/BOOTAA64.EFI"

    # kernel8-be.img — payload BE para UEFI+BE handoff (SCTLR_EL1.EE switch)
    if [ "$BIG_ENDIAN" -eq 1 ]; then
        if [ ! -f "$OUT_DIR/kernel8-be.img" ]; then
            echo "[ERROR] $OUT_DIR/kernel8-be.img não encontrado — deveria ter sido compilado antes"
            exit 1
        fi
        echo "[SD] + kernel8-be.img (BE payload, $(du -h "$OUT_DIR/kernel8-be.img" | cut -f1))"
        mcopy -i "$SDCARD_IMG" "$OUT_DIR/kernel8-be.img" "::kernel8-be.img"
    fi

    # config.txt — usa sdcard/config.txt se existir, senão gera default UEFI
    if [ -f "$SDCARD_OVERRIDES/config.txt" ]; then
        echo "[SD] config.txt: usando $SDCARD_OVERRIDES/config.txt"
        mcopy -i "$SDCARD_IMG" "$SDCARD_OVERRIDES/config.txt" "::config.txt"
    else
        cat > /tmp/rpi_config.txt << 'EOF'
arm_64bit=1
enable_uart=1
uart_2ndstage=1
init_uart_clock=48000000
EOF
        mcopy -i "$SDCARD_IMG" /tmp/rpi_config.txt "::config.txt"
        echo "[SD] config.txt: UEFI default (enable_uart=1, uart_2ndstage=1)"
    fi

    # Copia conteúdo de ./disks para a raiz da imagem
    local COPIED=0
    if [ -d "$DISKS_DIR" ]; then
        shopt -s nullglob dotglob
        local DISK_ITEMS=("$DISKS_DIR"/*)
        shopt -u dotglob
        if [ ${#DISK_ITEMS[@]} -gt 0 ]; then
            echo "[SD] Copiando conteúdo de $DISKS_DIR..."
            mcopy -i "$SDCARD_IMG" -s "${DISK_ITEMS[@]}" ::
            COPIED=${#DISK_ITEMS[@]}
        fi
        shopt -u nullglob
    fi

    echo ""
    echo "[SD] sdcard.img UEFI pronto! ($COPIED item(ns) copiado(s) de disks/)"
    echo "     Hardware: sudo dd if=sdcard.img of=/dev/sdX bs=4M status=progress && sync"
    echo ""
}

# ---------------------------------------------------------------------------
# Modo UEFI: build + SD card + sair (sem QEMU)
# ---------------------------------------------------------------------------

if [ "$UEFI_MODE" -eq 1 ]; then
    command -v cargo >/dev/null 2>&1 || { echo "[ERROR] cargo não encontrado"; exit 1; }

    cargo +nightly --version >/dev/null 2>&1 || {
        echo "[ERROR] cargo nightly não encontrado"
        echo "[HINT]  rustup toolchain install nightly"
        exit 1
    }

    # Modo UEFI+BE: compila kernel BE primeiro (payload), depois o loader LE
    if [ "$BIG_ENDIAN" -eq 1 ]; then
        echo "[BUILD] UEFI+BE: compilando kernel BE (payload)..."
        cargo +nightly build --release \
            --target "$TARGET_BE" \
            -Z build-std=core,compiler_builtins
        cargo +nightly objcopy --release \
            --target "$TARGET_BE" \
            -Z build-std=core,compiler_builtins \
            -- -O binary "$OUT_DIR/kernel8-be.img"
        echo "[BUILD] $OUT_DIR/kernel8-be.img pronto (payload BE)"
    fi

    echo "[BUILD] Compilando UEFI loader (LE — obrigatório pela spec AArch64 UEFI)..."
    cargo +nightly build --release \
        --target "$TARGET_UEFI" \
        -Z build-std=core,compiler_builtins

    cp "target/$TARGET_UEFI/release/raspi-labs.efi" "$EFI_OUT"
    echo "[BUILD] $EFI_OUT pronto"

    create_uefi_sdcard_image
    exit 0
fi

# ---------------------------------------------------------------------------
# Pré-requisitos (bare-metal)
# ---------------------------------------------------------------------------

command -v cargo >/dev/null 2>&1 || { echo "[ERROR] cargo não encontrado"; exit 1; }
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
    rm -f "$OUT_DIR/kernel8.img" "$OUT_DIR/kernel8-be.img"
    rm -f "$DTB_PATCHED"
fi

mkdir -p "$OUT_DIR"

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

# Cria a imagem SD somente depois que o kernel existir
if [ "$CREATE_SD" -eq 1 ]; then
    create_sdcard_image
fi

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
