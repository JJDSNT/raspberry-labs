# boot.cmd — script de boot U-Boot para desenvolvimento via TFTP
#
# Este arquivo é compilado para boot.scr com:
#   make boot-scr      (usa mkimage do pacote u-boot-tools)
#
# CONFIGURAÇÃO OBRIGATÓRIA: defina o IP do servidor TFTP abaixo.
# O servidor TFTP deve servir a pasta out/ do projeto.
#
# Workflow de desenvolvimento:
#   1. SD card com U-Boot fica gravado uma só vez (make sdcard-tftp)
#   2. Terminal A: sudo make tftp-server   (serve out/ via TFTP)
#   3. Terminal B: make le                 (compila novo kernel)
#   4. Reinicia o RPi — ele baixa o novo kernel automaticamente

# ---------------------------------------------------------------------------
# EDITE: IP do seu computador de desenvolvimento
# ---------------------------------------------------------------------------
setenv serverip 192.168.1.100

# ---------------------------------------------------------------------------
# Configurações do kernel (normalmente não precisa editar)
# ---------------------------------------------------------------------------
setenv kernel_addr  0x00080000
setenv bootfile     kernel8.img

# Argumento de boot padrão (pode ser sobrescrito via U-Boot interativo)
# Exemplos: "demo=audiotest" "demo=flame width=1280 height=720"
setenv bootargs "demo=flame"

# ---------------------------------------------------------------------------
# Boot
# ---------------------------------------------------------------------------
echo ""
echo "=== TFTP Boot — raspberry-labs ==="
echo "  Servidor : ${serverip}"
echo "  Arquivo  : ${bootfile}"
echo "  Demo     : ${bootargs}"
echo ""

# Inicializa USB (necessário para ethernet USB no RPi3B)
usb start

# Obtém IP via DHCP (opcional — comente se usar IP estático abaixo)
# Se o servidor DHCP não anunciar siaddr, serverip precisa estar definido acima.
dhcp

echo "  IP local : ${ipaddr}"
echo ""

# Baixa o kernel via TFTP
tftp ${kernel_addr} ${bootfile}

if test $? -ne 0; then
    echo "[ERRO] TFTP falhou. Verifique:"
    echo "  - serverip: ${serverip}"
    echo "  - sudo make tftp-server em execucao no PC"
    echo "  - Conectividade de rede"
    echo ""
    echo "Aguardando 10s antes de tentar novamente..."
    sleep 10
    reset
fi

echo ""
echo "=== Iniciando kernel em ${kernel_addr} ==="
go ${kernel_addr}
