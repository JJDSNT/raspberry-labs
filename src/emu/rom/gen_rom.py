#!/usr/bin/env python3
# gen_rom.py — extrai e converte o ROM AROS para headers C usados pelo Memory.c
#
# Uso:
#   cd src/emu/rom
#   python3 gen_rom.py
#
# Input:  aros.rom.cpp  (array C com dados gzip do ROM AROS)
# Output: ../c/omega2/aros_main.h  (512KB — carregado em 0xF80000)
#         ../c/omega2/aros_ext.h   (512KB — carregado em 0xE00000)

import re, gzip, os, sys

script_dir = os.path.dirname(os.path.abspath(__file__))
cpp_path   = os.path.join(script_dir, 'aros.rom.cpp')
rom_path   = os.path.join(script_dir, 'aros.rom')
out_dir    = os.path.join(script_dir, '..', 'c', 'omega2')

# --- Descomprime o ROM se necessário ---
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
parts = [('aros_ext',  raw[:half],  0xE00000),
         ('aros_main', raw[half:],  0xF80000)]

for name, rom_bytes, addr in parts:
    path = os.path.join(out_dir, f'{name}.h')
    with open(path, 'w') as f:
        f.write(f'/* {name}: {len(rom_bytes)} bytes — carregado em 0x{addr:06X} */\n')
        f.write(f'static const unsigned char {name}[] = {{\n')
        for i, b in enumerate(rom_bytes):
            f.write(f'0x{b:02x},')
            if (i + 1) % 16 == 0:
                f.write('\n')
        f.write('\n};\n')
    print(f'  {path}: {len(rom_bytes)} bytes @ 0x{addr:06X}')

print('OK — recompile para usar AROS (build.rs define USE_AROS)')
