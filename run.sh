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

usage() {
    echo "Usage: $0 [-b] [-c] [-C]"
    echo "  -b    build big-endian (requer cargo nightly)"
    echo "  -c    limpeza leve (remove kernel gerado e artefatos temporários)"
    echo "  -C    limpeza total (cargo clean)"
    exit 1
}

while getopts "bcCh" opt; do
    case $opt in
        b) BIG_ENDIAN=1 ;;
        c) CLEAN_LIGHT=1 ;;
        C) CLEAN_FULL=1 ;;
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
# Pré-requisitos
# ---------------------------------------------------------------------------

command -v cargo  >/dev/null 2>&1 || { echo "[ERROR] cargo não encontrado"; exit 1; }
command -v go     >/dev/null 2>&1 || { echo "[ERROR] go não encontrado"; exit 1; }
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
# DTB base — baixa uma vez para dtb/
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
# Launcher TUI
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
    go run .
)
