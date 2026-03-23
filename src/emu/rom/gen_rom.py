#!/usr/bin/env python3
# gen_rom.py — converte ROMs AROS para headers C usados pelo Memory.c
#
# Modo 1 — ROM ECS separados (preferido):
#   Coloque em src/arosrom/:
#     aros-amiga-m68k-rom.bin   (512KB — ROM principal → 0xF80000)
#     aros-amiga-m68k-ext.bin   (512KB — ROM extendida → 0xE00000)
#   Depois rode:
#     python3 src/emu/rom/gen_rom.py
#
# Modo 2 — ROM combinado AGA de 1MB (fallback):
#   Input:  aros.rom.cpp  (array C com dados gzip)
#   O script divide ao meio: primeira metade=ext (0xE00000), segunda=main (0xF80000)

import re, gzip, os, sys

script_dir = os.path.dirname(os.path.abspath(__file__))
root       = os.path.join(script_dir, '..', '..', '..')
rom_dir    = os.path.join(root, 'src', 'arosrom')
out_dir    = os.path.join(script_dir, '..', 'c', 'omega2', 'memory')

COLS = 16

def write_header(path, varname, addr, rom_bytes):
    with open(path, 'w') as f:
        f.write(f'/* {varname}: {len(rom_bytes)} bytes — carregado em 0x{addr:06X} */\n')
        f.write(f'static const unsigned char {varname}[] = {{\n')
        for i, b in enumerate(rom_bytes):
            f.write(f'0x{b:02x},')
            if (i + 1) % COLS == 0:
                f.write('\n')
        f.write('\n};\n')
    print(f'  {path}: {len(rom_bytes)} bytes @ 0x{addr:06X}')

# --- Modo 1: arquivos ECS separados ---
main_bin = os.path.join(rom_dir, 'aros-amiga-m68k-rom.bin')
ext_bin  = os.path.join(rom_dir, 'aros-amiga-m68k-ext.bin')

if os.path.exists(main_bin) and os.path.exists(ext_bin):
    print('Modo 1: usando ROMs ECS separados de src/arosrom/')
    with open(main_bin, 'rb') as f: main_bytes = f.read()
    with open(ext_bin,  'rb') as f: ext_bytes  = f.read()
    write_header(os.path.join(out_dir, 'aros_main.h'), 'aros_main', 0xF80000, main_bytes)
    write_header(os.path.join(out_dir, 'aros_ext.h'),  'aros_ext',  0xE00000, ext_bytes)
    print('OK — recompile com: cargo build --release')
    sys.exit(0)

# --- Modo 2: ROM combinado AGA de 1MB (fallback) ---
print('Modo 2: usando ROM combinado AGA (aros.rom.cpp)')
cpp_path = os.path.join(script_dir, 'aros.rom.cpp')
rom_path = os.path.join(script_dir, 'aros.rom')

if not os.path.exists(rom_path):
    print(f'Extraindo bytes de {cpp_path}...')
    with open(cpp_path) as f:
        content = f.read()
    hex_bytes = re.findall(r'0x([0-9a-fA-F]{2})', content)
    data = bytes(int(b, 16) for b in hex_bytes)
    if data[:2] != b'\x1f\x8b':
        print('ERRO: magic gzip não encontrado', file=sys.stderr)
        sys.exit(1)
    raw = gzip.decompress(data)
    with open(rom_path, 'wb') as f:
        f.write(raw)
    print(f'  {rom_path}: {len(raw)} bytes')
else:
    with open(rom_path, 'rb') as f:
        raw = f.read()
    print(f'Usando {rom_path}: {len(raw)} bytes')

if len(raw) != 1048576:
    print(f'AVISO: tamanho esperado 1048576, obtido {len(raw)}')

half = len(raw) // 2
write_header(os.path.join(out_dir, 'aros_ext.h'),  'aros_ext',  0xE00000, raw[:half])
write_header(os.path.join(out_dir, 'aros_main.h'), 'aros_main', 0xF80000, raw[half:])
print('OK — recompile com: cargo build --release')
