# sdcard/ — overrides para criação do SD card

Arquivos colocados aqui sobrescrevem os defaults gerados automaticamente
pelo `run.sh` ao criar `sdcard.img`.

## Arquivos suportados

| Arquivo        | Modo       | Comportamento                                                  |
|----------------|------------|----------------------------------------------------------------|
| `cmdline.txt`  | bare-metal | Kernel cmdline passada pelo bootloader (ex: `demo=flame`)     |
| `config.txt`   | ambos      | Boot config do RPi (substitui o gerado automaticamente)        |
| `RPI_EFI.fd`   | UEFI       | Firmware pftf/RPi3; alternativa a colocar em `firmware/`       |

## Exemplos de cmdline.txt

```
demo=flame
```
```
demo=omega rom=KS13.rom df0=wb13.adf
```
```
demo=plasma width=1280 height=720 depth=32
```

## Comportamento quando o arquivo está ausente

- **`cmdline.txt`**: usa `demo=flame`
- **`config.txt`**: gerado automaticamente com `enable_uart=1` e o `kernel=`
  correto para o modo (bare-metal LE/BE ou UEFI)
- **`RPI_EFI.fd`**: também procurado em `firmware/RPI_EFI.fd`; se não
  encontrado em nenhum lugar, o SD UEFI fica incompleto (aviso no log)

## Modos de criação

```sh
run.sh -s          # bare-metal LE  (kernel8.img)
run.sh -b -s       # bare-metal BE  (kernel8-be.img)
run.sh -u          # UEFI LE        (EFI/BOOT/BOOTAA64.EFI)
run.sh -u -b       # UEFI+BE        (BOOTAA64.EFI loader + kernel8-be.img payload)
```

Equivalentes via Make:

```sh
make sdcard
make sdcard-be
make sdcard-uefi
make sdcard-uefi-be
```
