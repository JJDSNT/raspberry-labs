# Arquitetura do projeto

## Visão geral

Kernel bare-metal para Raspberry Pi 3B em Rust (`no_std`, AArch64).
Objetivo: engine de demos estilo demoscene Amiga (Copper, Blitter, renderização software).
Objetivo de longo prazo: personalidade de kernel MC68k-64.

## Camadas

```
demos/          — demos individuais (implementam trait Demo)
gfx/            — renderer, blitter, copper, fonte, sprites, primitivas
audio/          — mixer de tons (síntese digital, N vozes)
math/           — math3d, superfícies, raytracer
media/          — relógio de mídia (MediaClock, FrameContext)
drivers/        — fachadas portáveis: audio, framebuffer, sdcard, uart, usb
  ↳ platform/raspi3/           — implementação específica RPi3
      peripheral/clock.rs      — clock manager BCM2837
      peripheral/dma.rs        — DMA controller
      peripheral/pcm.rs        — PCM/I2S periférico
      peripheral/sdhci.rs      — controlador eMMC/SD (Arasan SDHCI)
      peripheral/gpio.rs       — GPIO
      peripheral/uart.rs       — UART0
      peripheral/usb.rs        — USB (TinyUSB DWC2)
      audio.rs                 — orquestração PCM + DMA + clock
      mailbox.rs               — mailbox ARM↔VideoCore
kernel/         — scheduler, sync (IrqSafeSpinLock), time, console, tasks
arch/aarch64/   — boot, exceções, MMU, IRQ, SMP, timer
boot/           — boot_info, cmdline, entry point
fs/             — FAT32 read-only (usa drivers::sdcard)
emu/            — emulador Omega2 Amiga (Fase 1+2 completa)
uefi/           — caminho de boot UEFI
```

## Padrão de portabilidade

Cada subsistema tem duas camadas:
- `drivers/<subsistema>.rs` — fachada pública (única interface usada pelo kernel e demos)
- `platform/raspi3/<subsistema>.rs` — implementação concreta do BCM2837

O restante do kernel **nunca** importa diretamente de `platform/raspi3/`.

## Targets de build

| Target | Arquivo | Comando |
|--------|---------|---------|
| Bare-metal LE (padrão) | `out/kernel8.img` | `make le` |
| Bare-metal BE | `out/kernel8-be.img` | `make be` |
| UEFI | `out/BOOTAA64.EFI` | `make uefi` |

## Seleção de demo (cmdline)

O bootloader passa `cmdline.txt` para o kernel via DTB (`/chosen/bootargs`).
Exemplo: `demo=audiotest width=1280 height=720 depth=32`

Demos disponíveis: `audiotest`, `rasterbars`, `plasma`, `flame`, `starfield`,
`tunnel`, `parallax`, `juggler`, `sprite_bouncer`, `gfx3d_triangle`, `omega`.
Diagnósticos: `gradient`, `testpattern`, `smpte`.
