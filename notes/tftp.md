# TFTP Boot — desenvolvimento sem gravar SD card

## Visão geral

Com TFTP, o SD card do RPi fica permanente com U-Boot.
A cada mudança no kernel, basta compilar e reiniciar o RPi — ele
baixa o novo `kernel8.img` direto do computador de desenvolvimento via rede.

```
Computador de dev                RPi 3B
──────────────────               ──────────────────
make le                          Liga / reinicia
   → out/kernel8.img             
sudo make tftp-server ←── TFTP ──── U-Boot faz TFTP
                                     go 0x80000
                                     kernel executa
```

## Pré-requisitos

### No computador de desenvolvimento

```bash
sudo apt install u-boot-tools mtools
```

- `u-boot-tools`: para compilar `boot.scr` com `mkimage`
- `mtools`: para criar a imagem SD sem montar como root

### U-Boot para RPi3B 64-bit

O U-Boot precisa ser o build 64-bit para RPi3B.

**Opção 1 — Pacote Debian/Ubuntu** (mais simples):
```bash
sudo apt install u-boot-rpi
cp /usr/lib/u-boot/rpi_3/u-boot.bin firmware/u-boot.bin
```

**Opção 2 — Compilar do zero**:
```bash
git clone https://source.denx.de/u-boot/u-boot.git
cd u-boot
make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- rpi_3_defconfig
make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- -j$(nproc)
cp u-boot.bin /caminho/do/projeto/firmware/u-boot.bin
```

Requer: `gcc-aarch64-linux-gnu`
```bash
sudo apt install gcc-aarch64-linux-gnu
```

## Setup inicial (uma vez)

### 1. Configure o IP do servidor TFTP

Edite `scripts/boot.cmd` e altere:
```
setenv serverip 192.168.1.100   # ← coloque o IP do seu computador
```

### 2. Compile o boot script

```bash
make boot-scr
# gera out/boot.scr
```

### 3. Grave o SD card com U-Boot

```bash
make sdcard-tftp
# gera out/sdcard-tftp.img

# Grave no SD card (substitua sdX pelo seu dispositivo):
sudo dd if=out/sdcard-tftp.img of=/dev/sdX bs=4M status=progress && sync
```

Este SD card **não precisa ser gravado novamente** a cada build.

## Workflow de desenvolvimento

```bash
# Terminal A — servidor TFTP (fica rodando)
sudo make tftp-server

# Terminal B — ciclo de desenvolvimento
make le            # compila → out/kernel8.img
# reinicia o RPi — ele baixa e executa o novo kernel
```

### Trocar o demo pelo TFTP

O demo padrão é definido em `scripts/boot.cmd` via `setenv bootargs`.
Para trocar sem regravar o SD:

**Opção 1**: Interrompa o U-Boot durante o countdown e execute no prompt:
```
U-Boot> setenv bootargs "demo=audiotest"
U-Boot> tftp 0x80000 kernel8.img
U-Boot> go 0x80000
```

**Opção 2**: Edite `scripts/boot.cmd`, recompile e copie:
```bash
# Edite setenv bootargs em scripts/boot.cmd
make boot-scr
# Copie boot.scr para o SD card (sem regravar tudo):
sudo mount /dev/sdX /mnt
sudo cp out/boot.scr /mnt/
sudo umount /mnt
```

## Networking — WSL2

No WSL2, o servidor TFTP roda dentro do ambiente Linux mas o RPi precisa
alcançá-lo na rede física.

### WSL2 modo NAT (padrão)

O WSL2 fica em uma rede interna (172.x.x.x). O RPi na rede local não consegue
alcançá-lo diretamente.

**Solução**: port forwarding no Windows (PowerShell como Admin):
```powershell
# Descobrir IP do WSL2:
wsl hostname -I

# Criar regra de encaminhamento (substituir 172.x.x.x pelo IP do WSL2):
netsh interface portproxy add v4tov4 `
    listenport=69 listenaddress=0.0.0.0 `
    connectport=69 connectaddress=172.x.x.x

# Abrir no firewall:
netsh advfirewall firewall add rule name="TFTP" `
    protocol=UDP dir=in localport=69 action=allow

# Remover depois:
netsh interface portproxy delete v4tov4 listenport=69 listenaddress=0.0.0.0
```

Configure `serverip` em `scripts/boot.cmd` com o IP da máquina Windows
(não o IP do WSL2).

### WSL2 modo espelhado (Windows 11 22H2+)

Com networking espelhado, o WSL2 compartilha o IP da máquina Windows.
TFTP funciona diretamente sem port forwarding.

Ativar em `%USERPROFILE%\.wslconfig`:
```ini
[wsl2]
networkingMode=mirrored
```

## Porta sem root (6969)

Para não precisar de sudo no servidor TFTP:

```bash
TFTP_PORT=6969 make tftp-server-noroot
```

E no U-Boot antes do boot:
```
U-Boot> setenv tftpdstp 6969
U-Boot> tftp 0x80000 kernel8.img
U-Boot> go 0x80000
```

Ou adicione ao `boot.cmd`:
```
setenv tftpdstp 6969
```

## Arquivos relevantes

| Arquivo | Propósito |
|---------|-----------|
| `scripts/tftp-server.py` | Servidor TFTP Python (sem deps externas) |
| `scripts/boot.cmd` | Script U-Boot — edite `serverip` e `bootargs` |
| `scripts/mkbootscr.sh` | Compila boot.cmd → out/boot.scr |
| `firmware/u-boot.bin` | Binário U-Boot para RPi3B 64-bit (coloque aqui) |
| `out/boot.scr` | Script compilado (gerado por make boot-scr) |
| `out/sdcard-tftp.img` | Imagem SD com U-Boot (gerada por make sdcard-tftp) |
