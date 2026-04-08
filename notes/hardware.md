# Hardware — Raspberry Pi 3B / BCM2837

## Mapa de memória MMIO

| Periférico | ARM Physical | VC Bus |
|-----------|--------------|--------|
| Base MMIO | 0x3F000000 | 0x7E000000 |
| GPIO | 0x3F200000 | 0x7E200000 |
| UART0 | 0x3F201000 | 0x7E201000 |
| PCM/I2S | 0x3F203000 | 0x7E203000 |
| DMA (canais 0-14) | 0x3F007000 | 0x7E007000 |
| Clock manager | 0x3F101000 | 0x7E101000 |
| Mailbox | 0x3F00B880 | 0x7E00B880 |
| eMMC/SDHCI | 0x3F300000 | 0x7E300000 |
| USB (DWC2) | 0x3F980000 | 0x7E980000 |
| Local periph (timer, IRQ) | 0x40000000 | — |

## Conversão de endereços para DMA

```
bus_addr = physical | 0xC0000000   (uncached — sem cache coherency issues)
```

O DMA usa endereços no espaço de barramento do VideoCore.
Periféricos mapeados como `0x7Exxxxxx` no barramento VC.

## Canais DMA usados

| Canal | Uso |
|-------|-----|
| 2 | PCM/Áudio TX |

## Clocks (PLLD = 500 MHz fixo no RPi3)

| Clock | Freq | Uso |
|-------|------|-----|
| PLLD | 500 MHz | Fonte principal |
| PCM bit-clock | ~1.411 MHz | 44100 Hz × 32 clocks/frame |

## GPIO — Function Select

- GPIO 18: PCM_CLK (Alt5) — apenas para DAC externo, não para HDMI
- GPIO 19: PCM_FS (Alt5) — idem
- GPIO 40/41: PWM (Alt0) — áudio analógico 3.5mm (não implementado)

Para HDMI: nenhum GPIO de PCM necessário — o VideoCore lê diretamente do controlador PCM.

## SD Card

Controlador: Arasan SDHCI em `0x3F300000`.
Driver: `platform/raspi3/peripheral/sdhci.rs`.
Fachada: `drivers/sdcard.rs`.

## Mailbox (ARM ↔ VideoCore)

Canal 8 (Property): configura framebuffer, resolução, clocks de GPU, etc.
Usado por `drivers/framebuffer.rs` para alocar framebuffer via GPU.
Endereço enviado como `physical | 0xC0000000` (uncached VC view).
