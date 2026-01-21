# 🕹️ RPi3B Baremetal Amiga-Style Graphics

Exploração bare-metal do Raspberry Pi 3B focada em computação gráfica clássica e matemática volumétrica. O objetivo é transformar o Pi em uma "Amiga moderna", implementando truques clássicos de demoscene diretamente no metal.

## 🚀 O Projeto
Um motor gráfico escrito em **Rust** (bare-metal) que gerencia o hardware sem sistema operacional, focado em performance bruta e efeitos de varredura.

### 🎨 Core Engine (Estilo Amiga)
* **Software Copper:** Sincronia de raster para gradientes de fundo e mudanças de paleta por linha.
* **Software Blitter:** Manipulação de blocos de memória e transparências otimizadas via instruções **NEON**.
* **Framebuffer Driver:** Controle direto da GPU via Mailbox Interface para renderização de baixa latência.

### 📺 Demos & Efeitos
* **3D Scene:** Plotagem e rotação de funções volumétricas ($z = f(x, y)$) como paraboloides e campos de seno.
* **Oldschool FX:** Implementações clássicas de Flame, Plasma, Starfields e Scrolltext.
* **Raster Bars:** As famosas Copper bars que definiram a estética do Amiga.

---

## 🛠️ Stack Técnica
* **Linguagem:** Rust `no_std` + AArch64 Assembly.
* **Boot:** Nativo (`kernel8.img`) carregado em `0x80000`.
* **Target:** `aarch64-unknown-none-softfloat`.
* **Hardware:** Raspberry Pi 3B / 3B+.

## 📁 Estrutura do Código
```text
src/
├── boot.S        # Inicialização da CPU e setup da Stack
├── main.rs       # Orquestrador das Demos e loop principal
├── fb_driver.rs  # Negociação de Mailbox e controle do Framebuffer
├── blitter.rs    # Operações de bloco e transparência (NEON)
├── copper.rs     # Sincronia de raster e efeitos de varredura
├── math_3d.rs    # Projeção de perspectiva e rotação 3D
├── functions.rs  # Fórmulas das superfícies (z = x² + y², etc)
└── demos/        # Scripts das cenas individuais (flame, starfield, 3d)
```

## 🏁 Como Rodar
1.  **Build:** `cargo build --release`
2.  **Binário:** `rust-objcopy --strip-all -O binary target/.../amiga-blitter kernel8.img`
3.  **Deploy:** Copie o `kernel8.img` para um cartão SD com o firmware básico do RPi3.