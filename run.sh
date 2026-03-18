#!/bin/bash
set -e

KERNEL=kernel8.img
DTB_DIR="./dtb"
DTB="$DTB_DIR/bcm2710-rpi-3-b-plus.dtb"
DTB_URL="https://github.com/dhruvvyas90/qemu-rpi-kernel/raw/master/native-emulation/dtbs/bcm2710-rpi-3-b-plus.dtb"
LAUNCHER_DIR="./launch"
TARGET="aarch64-unknown-none-softfloat"

CLEAN=0

usage() {
    echo "Usage: $0 [-c]"
    echo "  -c    cargo clean antes de buildar"
    exit 1
}

while getopts "ch" opt; do
    case $opt in
        c) CLEAN=1 ;;
        h) usage ;;
        *) usage ;;
    esac
done

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
# Build
# ---------------------------------------------------------------------------

if [ "$CLEAN" -eq 1 ]; then
    echo "[BUILD] Limpando artefatos anteriores..."
    cargo clean
fi

echo "[BUILD] Compilando kernel..."
cargo build --release --target "$TARGET"

echo "[BUILD] Gerando $KERNEL..."
cargo objcopy --release --target "$TARGET" -- -O binary "$KERNEL"

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