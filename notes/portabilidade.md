# Portabilidade — guia para novos subsistemas

## Regra central

O código de `kernel/`, `demos/`, `gfx/`, `audio/` e `fs/`
**nunca importa diretamente de `platform/`**.
Só usa as fachadas em `drivers/`.

## Como adicionar um novo subsistema portável

### 1. Implementação de plataforma

Criar `src/platform/raspi3/peripheral/<nome>.rs` com o acesso MMIO.
Declarar em `src/platform/raspi3/peripheral/mod.rs`.

Se houver orquestração (vários periféricos), criar
`src/platform/raspi3/<nome>.rs` que usa os periféricos.
Declarar em `src/platform/raspi3/mod.rs`.

### 2. Fachada de driver

Criar `src/drivers/<nome>.rs` com funções `#[inline]` que delegam
para `crate::platform::raspi3::<nome>::*`.

Declarar em `src/drivers/mod.rs`.

### 3. Uso no kernel

Importar apenas `crate::drivers::<nome>` em qualquer lugar do kernel.

## Exemplo: áudio

```
platform/raspi3/peripheral/clock.rs    → Clock manager
platform/raspi3/peripheral/dma.rs      → DMA controller
platform/raspi3/peripheral/pcm.rs      → PCM/I2S periférico
platform/raspi3/audio.rs               → Orquestração dos três
drivers/audio.rs                       → Fachada pública
audio/mixer.rs                         → Síntese (independente de plataforma)
demos/audio_test.rs                    → Usa drivers::audio
```

## Exemplo: SD card

```
platform/raspi3/peripheral/sdhci.rs    → Arasan SDHCI
drivers/sdcard.rs                      → Fachada pública
fs/fat32.rs                            → Usa drivers::sdcard
```

## Gating UEFI vs bare-metal

Código exclusivo do path bare-metal: `#[cfg(not(target_os = "uefi"))]`
Código exclusivo UEFI: `#[cfg(target_os = "uefi")]`

Exemplos de itens gateados:
- `boot.S` — ponto de entrada bare-metal (`_start @ 0x80000`)
- `mod emu` — emulador Omega2
- `usb::init()` e `usb::handle_irq()` — USB via TinyUSB
- `mmu::init()` — MMU bare-metal
