# Audio — notas de implementação

## Hardware (BCM2837 / RPi3)

- **PCM/I2S** em `0x3F203000` — controlador de áudio serial
- **DMA canal 2** — streaming de RAM → FIFO PCM
- **Clock manager PCM** em `0x3F101098/9C` — PLLD ÷ 354.33 → 1.411 MHz bit-clock
- **HDMI audio** — o firmware VideoCore (start.elf) detecta atividade PCM e embute no stream HDMI

## Configuração de clock

```
PLLD = 500 MHz (fixo no RPi3)
bit-clock alvo = 44100 Hz × 32 clocks/frame = 1 411 200 Hz
divisor = 500 000 000 / 1 411 200 = 354.33
DIVI = 354, DIVF = 338  (338/1024 × 500MHz ≈ parte fracionária)
MASH = 1 (suaviza a fração)
```

## Formato de frame PCM

```
Frame: 32 clocks total
CH1 (Left) : clocks 1–16   (pos=1,  wid=8 → 16 bits)
CH2 (Right): clocks 17–32  (pos=17, wid=8 → 16 bits)
```

## Formato de buffer DMA

```
[L0, R0, L1, R1, ...]   — STEREO_WORDS = BUFFER_SAMPLES × 2 entradas u32
Cada u32: amostra i16 nos 16 bits inferiores
BUFFER_SAMPLES = 1024  → ~23ms por buffer @ 44100 Hz
```

## DMA ping-pong

```
CB[0]: src=bus(buf0), dst=0x7E203004 (FIFO), next=bus(CB[1])
CB[1]: src=bus(buf1), dst=0x7E203004,        next=bus(CB[0])
TI = SRC_INC | DEST_DREQ | WAIT_RESP | PERMAP=2 (PCM TX)
bus_addr(ptr) = (ptr as u32) | 0xC000_0000  (uncached VC alias)
```

## Arquivos

| Arquivo | Responsabilidade |
|---------|-----------------|
| `drivers/audio.rs` | Fachada pública — única interface usada pelos demos |
| `platform/raspi3/audio.rs` | Orquestração: init, playing_buffer, fill_back_buffer |
| `platform/raspi3/peripheral/clock.rs` | Clock manager — configure_pcm_for_44k1_x_32fs() |
| `platform/raspi3/peripheral/dma.rs` | DMA controller — bus_addr, DmaCb, canal ops |
| `platform/raspi3/peripheral/pcm.rs` | PCM periférico — configure_default_tx(), enable_tx_with_dma() |
| `audio/mixer.rs` | Síntese digital — tabela de seno 256 entradas, N_VOICES=4 |
| `demos/audio_test.rs` | Demo: osciloscópio visual + sequenciador de notas |

## config.txt obrigatório para áudio HDMI

```
hdmi_drive=2            # força HDMI com áudio (sem isso o RPi pode usar DVI = sem áudio)
hdmi_ignore_edid_audio=1  # ignora restrições de áudio anunciadas pelo monitor
```

Já adicionado em `sdcard/config.txt`.

## Launcher

Demo disponível no launcher TUI como `Audio Test` (bootarg: `audiotest`).
Já adicionado em `launch/demo/demos.txt`.
