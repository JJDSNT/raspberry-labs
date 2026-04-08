#!/usr/bin/env python3
"""
tftp-server.py — servidor TFTP read-only para desenvolvimento bare-metal.

Sem dependências externas. Protocolo RFC 1350.
Por padrão serve a pasta out/ na raiz do projeto.

Uso:
    sudo python3 scripts/tftp-server.py              # porta 69 (padrão)
    TFTP_PORT=6969 python3 scripts/tftp-server.py    # sem root, porta alta
    TFTP_ROOT=/outro/dir python3 scripts/tftp-server.py
"""

import os
import socket
import struct
import threading
import time
import sys
import subprocess

# ---------------------------------------------------------------------------
# Configuração
# ---------------------------------------------------------------------------

PORT      = int(os.environ.get("TFTP_PORT", "69"))
ROOT      = os.path.realpath(os.environ.get("TFTP_ROOT", "out"))
TIMEOUT   = float(os.environ.get("TFTP_TIMEOUT", "3"))
RETRIES   = int(os.environ.get("TFTP_RETRIES", "5"))

BLOCK_SIZE = 512

# ---------------------------------------------------------------------------
# Opcodes TFTP (RFC 1350)
# ---------------------------------------------------------------------------

OP_RRQ   = 1   # Read Request
OP_WRQ   = 2   # Write Request (recusado)
OP_DATA  = 3   # Data
OP_ACK   = 4   # Acknowledgement
OP_ERROR = 5   # Error

ERR_NOT_FOUND    = 1
ERR_ACCESS       = 2
ERR_ILLEGAL      = 4

# ---------------------------------------------------------------------------
# Helpers de pacote
# ---------------------------------------------------------------------------

def make_data(block: int, chunk: bytes) -> bytes:
    return struct.pack("!HH", OP_DATA, block) + chunk

def make_error(code: int, msg: str) -> bytes:
    return struct.pack("!HH", OP_ERROR, code) + msg.encode() + b"\x00"

def parse_rrq(data: bytes) -> tuple[str, str]:
    """Extrai (filename, mode) de um pacote RRQ."""
    parts = data[2:].split(b"\x00")
    filename = parts[0].decode("ascii", errors="replace")
    mode     = parts[1].decode("ascii", errors="replace").lower() if len(parts) > 1 else "octet"
    return filename, mode

# ---------------------------------------------------------------------------
# Transferência individual (thread por cliente)
# ---------------------------------------------------------------------------

def transfer(client_addr: tuple, filename: str):
    # Valida que o arquivo está dentro do root (evita path traversal)
    path = os.path.realpath(os.path.join(ROOT, filename))
    if not path.startswith(ROOT + os.sep) and path != ROOT:
        print(f"[TFTP] DENIED  {filename} (fora do root)")
        return

    # Socket efêmero para esta transferência
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.settimeout(TIMEOUT)
    sock.bind(("", 0))

    try:
        with open(path, "rb") as f:
            data = f.read()
    except FileNotFoundError:
        sock.sendto(make_error(ERR_NOT_FOUND, "File not found"), client_addr)
        print(f"[TFTP] 404     {filename}")
        sock.close()
        return
    except PermissionError:
        sock.sendto(make_error(ERR_ACCESS, "Access denied"), client_addr)
        print(f"[TFTP] DENIED  {filename}")
        sock.close()
        return

    total   = len(data)
    n_blocks = max(1, (total + BLOCK_SIZE - 1) // BLOCK_SIZE)
    sent    = 0
    t0      = time.monotonic()

    for blk in range(1, n_blocks + 1):
        chunk  = data[(blk - 1) * BLOCK_SIZE : blk * BLOCK_SIZE]
        packet = make_data(blk, chunk)

        for attempt in range(RETRIES):
            sock.sendto(packet, client_addr)
            try:
                ack, addr = sock.recvfrom(4)
                op, ack_blk = struct.unpack("!HH", ack[:4])
                if op == OP_ACK and ack_blk == blk:
                    sent += len(chunk)
                    break
                if op == OP_ERROR:
                    print(f"[TFTP] ERROR   {filename}: cliente enviou erro")
                    return
            except socket.timeout:
                if attempt == RETRIES - 1:
                    print(f"[TFTP] TIMEOUT {filename} (bloco {blk})")
                    return

    elapsed = time.monotonic() - t0
    kbps    = (total / 1024) / elapsed if elapsed > 0 else 0
    print(f"[TFTP] OK      {filename}  {total:,} bytes  {kbps:.0f} KB/s  {client_addr[0]}")
    sock.close()

# ---------------------------------------------------------------------------
# Loop principal
# ---------------------------------------------------------------------------

def show_ips():
    """Mostra IPs locais disponíveis."""
    try:
        out = subprocess.check_output(["ip", "-4", "addr", "show"], text=True)
        for line in out.splitlines():
            line = line.strip()
            if line.startswith("inet ") and "127." not in line:
                ip = line.split()[1].split("/")[0]
                print(f"         {ip}")
    except Exception:
        try:
            hostname = socket.gethostname()
            ip = socket.gethostbyname(hostname)
            print(f"         {ip}")
        except Exception:
            print("         (não foi possível determinar o IP)")

def main():
    if not os.path.isdir(ROOT):
        print(f"[ERRO] Diretório root '{ROOT}' não existe.")
        print(f"       Execute 'make le' primeiro para gerar out/kernel8.img")
        sys.exit(1)

    print("=" * 60)
    print(f"  TFTP Server — raspberry-labs")
    print(f"  Root : {ROOT}")
    print(f"  Porta: {PORT}")
    print(f"  IPs locais:")
    show_ips()
    if PORT < 1024:
        print(f"")
        print(f"  Configure U-Boot:")
        print(f"    setenv serverip <IP-acima>")
    print("=" * 60)

    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)

    try:
        sock.bind(("", PORT))
    except PermissionError:
        print(f"\n[ERRO] Porta {PORT} requer root.")
        print(f"       Execute com sudo, ou use uma porta alta:")
        print(f"         TFTP_PORT=6969 python3 scripts/tftp-server.py")
        sys.exit(1)

    print(f"[TFTP] Aguardando requisições...")
    print(f"       (Ctrl+C para parar)\n")

    try:
        while True:
            data, addr = sock.recvfrom(516)
            if len(data) < 2:
                continue

            op = struct.unpack("!H", data[:2])[0]

            if op == OP_RRQ:
                filename, mode = parse_rrq(data)
                print(f"[TFTP] RRQ     {filename}  de {addr[0]}")
                t = threading.Thread(
                    target=transfer,
                    args=(addr, filename),
                    daemon=True,
                )
                t.start()

            elif op == OP_WRQ:
                # Não suportamos escrita
                err = make_error(ERR_ACCESS, "Server is read-only")
                sock.sendto(err, addr)

    except KeyboardInterrupt:
        print("\n[TFTP] Servidor parado.")
    finally:
        sock.close()

if __name__ == "__main__":
    main()
