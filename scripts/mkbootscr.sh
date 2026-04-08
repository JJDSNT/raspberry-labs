#!/bin/bash
# mkbootscr.sh — compila scripts/boot.cmd → out/boot.scr
#
# Requer: mkimage (pacote u-boot-tools)
#   sudo apt install u-boot-tools

set -e
cd "$(dirname "$0")/.."

if ! command -v mkimage >/dev/null 2>&1; then
    echo "[ERRO] mkimage não encontrado."
    echo "       sudo apt install u-boot-tools"
    exit 1
fi

mkdir -p out
mkimage -C none -A arm64 -T script -d scripts/boot.cmd out/boot.scr

echo "[OK] out/boot.scr gerado"
echo "     Copie para o SD card junto com u-boot.bin (make sdcard-tftp)"
