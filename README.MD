# RPi3B Baremetal Amiga-Style Graphics

Exploração bare-metal do Raspberry Pi 3B inspirada nos gráficos clássicos do Amiga.  
Objetivo: criar efeitos gráficos no framebuffer, simulando *blitter*, *copper* e outros truques clássicos, do boot ao frame final — tudo sem sistema operacional.

- **Boot via UEFI** (compatível com Pi 3B+ firmware open source)
- **Acesso ao framebuffer** (sem Linux, direto no metal)
- **Blitter/Copper por software** (exemplos didáticos e evolutivos)
- **Código em C (e opcionalmente Rust)**

> **Aviso**: Projeto experimental e didático. Não recomendado para produção.

## Estrutura

- `src/` - Códigos-fonte dos efeitos, inicialização, blitter, copper, etc.
- `docs/` - Documentação técnica e referências
- `build/` - Binários gerados
- `tools/` - Scripts auxiliares

## Como rodar

1. Compile com o toolchain ARM (ex: GCC ARM)
2. Gere binário UEFI (`.efi`) para boot na firmware UEFI do Pi 3B+
3. Copie para SD (partição FAT32) e faça o boot

Veja `docs/uefi-boot.md` para detalhes de configuração do firmware.

## Roadmap

- [ ] Inicializar framebuffer e preencher a tela
- [ ] Implementar "blitter" para cópia rápida de blocos
- [ ] Simular copper/raster effects
- [ ] Efeitos clássicos (scrolltext, barras, plasma, etc)
- [ ] (Opcional) Portar para Rust ou Assembly


# 🧪 raspberry-labs

Laboratório de experimentos bare-metal e low-level no Raspberry Pi 3B.

Aqui você encontra projetos de baixo nível para Raspberry Pi, explorando gráficos, boot via UEFI, manipulação direta de hardware, e inspirações em clássicos como o Amiga.

## Projetos

- [`amiga-blitter/`](amiga-blitter/) — Efeitos gráficos no framebuffer inspirados no Amiga (blitter, copper, raster).
- [`uefi-boot/`](uefi-boot/) — Exemplos de inicialização do Raspberry Pi via UEFI.
- [`framebuffer-fx/`](framebuffer-fx/) — Experimentos de manipulação direta do framebuffer.
- [`gpio-tests/`](gpio-tests/) — Testes básicos com GPIO, LEDs e sensores.

## Objetivo

Reunir estudos, exemplos e ferramentas para quem quer aprender ou experimentar o Raspberry Pi além do Linux, direto no metal.

> Projeto aberto, educacional e em evolução contínua.  
> Contribuições e forks são bem-vindos!

---

## Como começar

Cada subprojeto possui seu próprio README, explicando requisitos, compilação e execução.

```code
raspberry-labs/
  amiga-blitter/
  framebuffer-fx/
  uefi-boot/
  gpio-tests/
  docs/
  tools/
```

## Licença

MIT, exceto onde indicado.

---
