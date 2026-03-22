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
CREATE_RPI=0

DISKS_DIR="./disks"
SD_IMG="$(pwd)/sd.img"
SDCARD_IMG="$(pwd)/sdcard.img"
FIRMWARE_DIR="./firmware"

# Firmware oficial RPi (bootado direto no hardware, não no QEMU)
FIRMWARE_BASE="https://github.com/raspberrypi/firmware/raw/master/boot"
FIRMWARE_FILES="bootcode.bin start.elf fixup.dat bcm2710-rpi-3-b-plus.dtb"

usage() {
    echo "Usage: $0 [-b] [-c] [-C] [-s] [-r]"
    echo "  -b    build big-endian (requer cargo nightly)"
    echo "  -c    limpeza leve (remove kernel gerado e artefatos temporários)"
    echo "  -C    limpeza total (cargo clean)"
    echo "  -s    cria/atualiza sd.img para uso no QEMU"
    echo "  -r    build + cria sdcard.img para gravar no SD card físico do RPi"
    exit 1
}

while getopts "bcCsrh" opt; do
    case $opt in
        b) BIG_ENDIAN=1 ;;
        c) CLEAN_LIGHT=1 ;;
        C) CLEAN_FULL=1 ;;
        s) CREATE_SD=1 ;;
        r) CREATE_RPI=1 ;;
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
# SD card QEMU (sd.img — FAT32 sem partição, para -sd do QEMU)
# ---------------------------------------------------------------------------

create_sd_image() {
    command -v mcopy >/dev/null 2>&1 || {
        echo "[ERROR] mtools não encontrado"
        echo "[HINT]  sudo apt install mtools"
        exit 1
    }

    mkdir -p "$DISKS_DIR"

    echo "[SD] Criando $SD_IMG (64 MB, FAT32)..."
    dd if=/dev/zero of="$SD_IMG" bs=1M count=64 status=none
    mformat -i "$SD_IMG" -F -v "OMEGA" ::

    ADDED=0
    for n in 0 1; do
        ADF="$DISKS_DIR/disk${n}.adf"
        if [ -f "$ADF" ]; then
            echo "[SD] Adicionando disk${n}.adf ($(du -h "$ADF" | cut -f1))..."
            mcopy -i "$SD_IMG" "$ADF" "::disk${n}.adf"
            ADDED=$((ADDED + 1))
        else
            echo "[SD] disk${n}.adf não encontrado em $DISKS_DIR — slot vazio"
        fi
    done

    echo "[SD] $SD_IMG pronto ($ADDED disco(s) adicionado(s))"
}

# ---------------------------------------------------------------------------
# SD card físico RPi (sdcard.img — MBR + FAT32 com firmware de boot)
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

create_rpi_image() {
    command -v sfdisk >/dev/null 2>&1 || {
        echo "[ERROR] sfdisk não encontrado (util-linux)"
        exit 1
    }
    command -v mcopy >/dev/null 2>&1 || {
        echo "[ERROR] mtools não encontrado"
        echo "[HINT]  sudo apt install mtools"
        exit 1
    }

    # Parâmetros da imagem
    local SIZE_MB=256
    local PART_START=8192                           # sectores (offset 4MB — padrão RPi)
    local PART_OFFSET=$(( PART_START * 512 ))       # bytes
    local TOTAL_SECTORS=$(( SIZE_MB * 1024 * 1024 / 512 ))
    local PART_SECTORS=$(( TOTAL_SECTORS - PART_START ))

    download_firmware

    echo "[RPI] Criando $SDCARD_IMG (${SIZE_MB}MB)..."
    dd if=/dev/zero of="$SDCARD_IMG" bs=1M count="$SIZE_MB" status=none

    # Tabela MBR: partição FAT32 LBA bootável
    sfdisk --quiet "$SDCARD_IMG" << EOF
label: dos
unit: sectors

1 : start=${PART_START}, size=${PART_SECTORS}, type=c, bootable
EOF

    # Formata FAT32 na partição
    mformat -i "${SDCARD_IMG}@@${PART_OFFSET}" -F -T "$PART_SECTORS" -v "RASPI" ::

    # Firmware RPi
    for f in $FIRMWARE_FILES; do
        if [ -f "$FIRMWARE_DIR/$f" ]; then
            echo "[RPI] + $f"
            mcopy -i "${SDCARD_IMG}@@${PART_OFFSET}" "$FIRMWARE_DIR/$f" ::
        fi
    done

    # Kernel
    echo "[RPI] + kernel8.img ($(du -h "$KERNEL" | cut -f1))"
    mcopy -i "${SDCARD_IMG}@@${PART_OFFSET}" "$KERNEL" "::kernel8.img"

    # config.txt — boot AArch64 sem modificações do firmware
    cat > /tmp/rpi_config.txt << 'EOF'
arm_64bit=1
kernel=kernel8.img
# disable_overscan=1
EOF
    mcopy -i "${SDCARD_IMG}@@${PART_OFFSET}" /tmp/rpi_config.txt "::config.txt"

    # cmdline.txt — bootargs lidos pelo firmware e injetados no DTB /chosen/bootargs
    # Escolhe demo automaticamente: omega se discos disponíveis, flame caso contrário
    local CMDLINE="demo=flame"
    if [ -f "$DISKS_DIR/disk0.adf" ]; then
        CMDLINE="demo=omega df0=disk0.adf df1=disk1.adf"
        # Adiciona ROM se encontrada no diretório de discos
        for ROM_FILE in "$DISKS_DIR"/*.rom "$DISKS_DIR"/*.ROM; do
            [ -f "$ROM_FILE" ] || continue
            ROM_NAME="$(basename "$ROM_FILE")"
            CMDLINE="$CMDLINE rom=$ROM_NAME"
            echo "[RPI] ROM: $ROM_NAME"
            mcopy -i "${SDCARD_IMG}@@${PART_OFFSET}" "$ROM_FILE" "::$ROM_NAME"
            break  # apenas o primeiro ROM encontrado
        done
    fi
    printf '%s' "$CMDLINE" > /tmp/rpi_cmdline.txt
    mcopy -i "${SDCARD_IMG}@@${PART_OFFSET}" /tmp/rpi_cmdline.txt "::cmdline.txt"
    echo "[RPI] cmdline: $CMDLINE"

    # ADFs (se existirem)
    for n in 0 1; do
        local ADF="$DISKS_DIR/disk${n}.adf"
        if [ -f "$ADF" ]; then
            echo "[RPI] + disk${n}.adf ($(du -h "$ADF" | cut -f1))"
            mcopy -i "${SDCARD_IMG}@@${PART_OFFSET}" "$ADF" "::disk${n}.adf"
        fi
    done

    echo ""
    echo "[RPI] sdcard.img pronto!"
    echo ""
    echo "      Grave com:"
    echo "        sudo dd if=sdcard.img of=/dev/sdX bs=4M status=progress && sync"
    echo "      ou use Balena Etcher (https://etcher.balena.io)"
    echo ""
    echo "      Para mudar o demo padrão edite firmware/cmdline.txt"
    echo "      e rode ./run.sh -r novamente."
}

if [ "$CREATE_SD" -eq 1 ]; then
    create_sd_image
fi

# ---------------------------------------------------------------------------
# Pré-requisitos
# ---------------------------------------------------------------------------

command -v cargo  >/dev/null 2>&1 || { echo "[ERROR] cargo não encontrado"; exit 1; }

# Para -r não é necessário o launcher Go nem fdtput
if [ "$CREATE_RPI" -eq 0 ]; then
    command -v go >/dev/null 2>&1 || { echo "[ERROR] go não encontrado"; exit 1; }
    command -v fdtput >/dev/null 2>&1 || {
        echo "[ERROR] fdtput não encontrado"
        echo "[HINT]  sudo apt install device-tree-compiler"
        exit 1
    }
fi

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

if [ "$CREATE_RPI" -eq 0 ]; then
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
# Imagem RPi físico (sai após criar, sem launcher)
# ---------------------------------------------------------------------------

if [ "$CREATE_RPI" -eq 1 ]; then
    create_rpi_image
    exit 0
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
    SD_IMG_PATH="$SD_IMG" \
    go run .
)
