DTBs para emulação QEMU
=======================

Este diretório contém Device Tree Blobs específicos para o QEMU.

Arquivos
--------
bcm2710-rpi-3-b-plus.dtb        (baixado automaticamente pelo run.sh)
  Origem: https://github.com/dhruvvyas90/qemu-rpi-kernel
  Versão modificada para funcionar com a emulação QEMU do RPi 3 B+.
  Não é o DTB oficial — tem patches para compatibilidade com o emulador.

bcm2710-rpi-3-b-plus-patched.dtb  (gerado em tempo de execução, não versionado)
  Criado pelo run.sh/launcher a partir do base acima com fdtput,
  injetando o cmdline (bootargs) escolhido na TUI antes de passar ao QEMU.

bcm2710-rpi-3-b-plus-test.dtb     (gerado durante testes, não versionado)


Não confundir com firmware/
---------------------------
A pasta firmware/ contém o DTB oficial da Raspberry Pi Foundation:
  Origem: https://github.com/raspberrypi/firmware/raw/master/boot
  Tamanho diferente (35 KB vs 28 KB), conteúdo diferente.
  Usado exclusivamente para o cartão SD / hardware real.
  Não funciona corretamente no QEMU e vice-versa.
