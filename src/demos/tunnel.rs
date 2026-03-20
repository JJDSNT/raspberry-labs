// src/demos/tunnel.rs
//
// Free-directional Tunnel — portado do original Amiga de CorTeX/Optimum (1998)
//
// Algoritmo fiel ao original:
//   1. do_precalc: calcula coordenadas exatas (alpha, z) numa grade W/8 × H/8
//      usando interseção raio-cilindro + atan2 em coordenadas esféricas.
//   2. refresh: interpola entre os pontos da grade com fixed-point 16.16,
//      fazendo lookup na textura 128×128.
//
// Assets embutidos no binário:
//   - texture_rotated.bin : 128×128 bytes, índices de textura (já rotacionada)
//   - palette.bin         : 256 × u32 ARGB8888 little-endian
//
// Coloque os dois arquivos em src/demos/ e ajuste os paths do include_bytes!
// se necessário.

use crate::demos::Demo;
use crate::gfx::renderer::Renderer;
use crate::media::FrameContext;

// Assets embutidos — ajuste o path relativo ao Cargo.toml se necessário
static TEXTURE_DATA: &[u8] = include_bytes!("texture_rotated.bin");
static PALETTE_DATA: &[u8] = include_bytes!("palette.bin");

const TEX_SIZE: usize = 128;
const RADIUS:   f32   = 64.0;
const _DIST:    f32   = 256.0;

// Grade de pré-cálculo: (H/8+1) × (W/8+1) — mesmas dimensões do original.
// Usamos dimensões máximas fixas para evitar alocação dinâmica.
// Tamanho máximo da grade — baseado na resolução máxima do renderer (1024×768).
// A grade real usada em runtime é (w/8+1) × (h/8+1), calculada por frame.
// Esses valores cobrem qualquer resolução que o bootargs possa passar.
const GRID_W: usize = crate::gfx::renderer::MAX_WIDTH  / 8 + 1; // 129 para 1024px
const GRID_H: usize = crate::gfx::renderer::MAX_HEIGHT / 8 + 1; //  97 para 768px

pub struct TunnelDemo {
    /// Coordenada angular no cilindro (eixo U da textura)
    alpha: [[f32; GRID_W]; GRID_H],
    /// Profundidade no cilindro (eixo V da textura)
    zede:  [[f32; GRID_W]; GRID_H],

    /// Paleta ARGB32 — 256 entradas lidas do palette.bin
    palette: [u32; 256],

    // Estado da câmera — idêntico às variáveis do action() original
    aa:     f32,  // alpha esférico da direção de visada
    thet:   f32,  // theta esférico
    eaa:    f32,  // rotação do plano de tela
    dalpha: f32,  // offset angular do observador no cilindro
}

impl TunnelDemo {
    pub fn new() -> Self {
        // Carrega paleta do slice estático (256 × 4 bytes, little-endian u32)
        let mut palette = [0u32; 256];
        for i in 0..256 {
            let off = i * 4;
            palette[i] = u32::from_le_bytes([
                PALETTE_DATA[off],
                PALETTE_DATA[off + 1],
                PALETTE_DATA[off + 2],
                PALETTE_DATA[off + 3],
            ]);
        }

        Self {
            alpha:   [[0.0; GRID_W]; GRID_H],
            zede:    [[0.0; GRID_W]; GRID_H],
            palette,
            aa:     0.0,
            thet:   0.0,
            eaa:    0.0,
            dalpha: 0.0,
        }
    }

    // -----------------------------------------------------------------------
    // do_precalc — tradução direta do C original
    // -----------------------------------------------------------------------
    //
    // Calcula alpha[][] e zede[][] para cada ponto da grade W/8 × H/8.
    // Parâmetros idênticos ao original:
    //   xd, z_off, dalpha : posição do observador no cilindro
    //   (cx,cy,cz)        : vetor de visada central
    //   (vx1,vy1,vz1)     : vetor X-tela
    //   (vx2,vy2,vz2)     : vetor Y-tela

    #[allow(clippy::too_many_arguments)]
    fn do_precalc(
        &mut self,
        w: usize, h: usize,
        xd: f32, z_off: f32, dalpha: f32,
        cx: f32, cy: f32, cz: f32,
        mut vx1: f32, mut vy1: f32, mut vz1: f32,
        vx2: f32, vy2: f32, vz2: f32,
    ) {
        let gw = w / 8 + 1;
        let gh = h / 8 + 1;

        let prec3 = xd * xd - 1.0;

        // Normaliza vx1 pela largura da grade (igual ao C: vx1 /= W/8.0)
        let inv_gw = 1.0 / (w as f32 / 8.0);
        vx1 *= inv_gw;
        vy1 *= inv_gw;
        vz1 *= inv_gw;

        for j in 0..gh {
            let jf = j as f32;
            let hf = h as f32;
            let mut x = cx - vx1 * 4.0 / w as f32 + (jf - hf / 16.0) * 8.0 * vx2 / hf;
            let mut y = cy - vy1 * 4.0 / w as f32 + (jf - hf / 16.0) * 8.0 * vy2 / hf;
            let mut z = cz - vz1 * 4.0 / w as f32 + (jf - hf / 16.0) * 8.0 * vz2 / hf;

            for i in 0..gw {
                x += vx1;
                y += vy1;
                z += vz1;

                let prec1 = x * xd;
                let prec2 = y * y + x * x;

                if prec2 >= 0.00001 {
                    let t = (-prec1 + libm_sqrt(prec1 * prec1 - prec3 * prec2)) / prec2;

                    self.alpha[j][i] = (libm_atan2(t * y, xd + t * x) + dalpha)
                        * 128.0 / core::f32::consts::PI;
                    self.zede[j][i] = z_off + 8.0 * t * z;
                } else {
                    self.alpha[j][i] = 0.0;
                    self.zede[j][i] = 1000.0;
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // refresh — tradução direta do C original
    // -----------------------------------------------------------------------
    //
    // Interpola entre os pontos da grade com fixed-point 16.16 e faz lookup
    // na textura. Saída: buffer ARGB32 via renderer.back_buffer().

    fn refresh(&self, renderer: &mut Renderer, w: usize, h: usize) {
        let buf = renderer.back_buffer();

        for j in 0..(h / 8) {
            for i in 0..(w / 8) {
                // Quatro cantos do bloco 8×8
                let al0 = self.alpha[j][i];
                let al1 = Self::fix_seam(self.alpha[j + 1][i],     al0);
                let al2 = Self::fix_seam(self.alpha[j][i + 1],     al0);
                let al3 = Self::fix_seam(self.alpha[j + 1][i + 1], al2);

                // Fixed-point 16.16
                let mut a0  = (al0 * 65536.0) as i32;
                let     a0e = (al1 * 65536.0) as i32;
                let mut z0  = (self.zede[j][i]         * 65536.0) as i32;
                let     z0e = (self.zede[j + 1][i]     * 65536.0) as i32;

                let mut a1  = (al2 * 65536.0) as i32;
                let     a1e = (al3 * 65536.0) as i32;
                let mut z1  = (self.zede[j][i + 1]     * 65536.0) as i32;
                let     z1e = (self.zede[j + 1][i + 1] * 65536.0) as i32;

                // Incrementos por linha (÷8)
                let a0i = (a0e - a0) / 8;
                let z0i = (z0e - z0) / 8;
                let a1i = (a1e - a1) / 8;
                let z1i = (z1e - z1) / 8;

                for jj in 0..8usize {
                    let row_start = (j * 8 + jj) * w + i * 8;

                    // Incrementos por pixel (÷8)
                    let ai = (a1 - a0) >> 3;
                    let zi = (z1 - z0) >> 3;

                    let mut a = a0;
                    let mut z = z0;

                    for ii in 0..8usize {
                        // Lookup na textura — idêntico ao C:
                        // ((a>>17)&127) | ((z>>8) & (127<<7))
                        let tex_u = (a >> 17) & 127;
                        let tex_v = (z >>  8) & (127 << 7);
                        let tex_idx = (tex_u | tex_v) as usize;

                        let pal_idx = TEXTURE_DATA[tex_idx & (TEX_SIZE * TEX_SIZE - 1)] as usize;
                        buf[row_start + ii] = self.palette[pal_idx];

                        a += ai;
                        z += zi;
                    }

                    a0 += a0i;
                    z0 += z0i;
                    a1 += a1i;
                    z1 += z1i;
                }
            }
        }
    }

    // Corrige a junção entre as duas extremidades da textura (0 = 2π).
    // Idêntico ao kludge do original: se |al - al0| > 100, usa al0.
    #[inline]
    fn fix_seam(al: f32, al0: f32) -> f32 {
        if (al - al0).abs() > 100.0 { al0 } else { al }
    }
}

impl Demo for TunnelDemo {
    fn render(&mut self, renderer: &mut Renderer, _frame: &FrameContext) {
        // Clamp à resolução máxima suportada pela grade
        let w = renderer.width().min((GRID_W - 1) * 8);
        let h = renderer.height().min((GRID_H - 1) * 8);
        if w == 0 || h == 0 { return; }

        // --- Atualiza câmera (idêntico ao action() do original) ---

        // Vetores do plano de tela em coordenadas esféricas
        let exx = -libm_sin(self.aa);
        let exy =  libm_cos(self.aa);
        let exz =  0.0_f32;
        let eyx = -libm_cos(self.aa) * libm_sin(self.thet);
        let eyy = -libm_sin(self.aa) * libm_sin(self.thet);
        let eyz =  libm_cos(self.thet);

        // Escala ×2 (igual ao original)
        let (exx, exy, exz) = (exx * 2.0, exy * 2.0, exz * 2.0);
        let (eyx, eyy, eyz) = (eyx * 2.0, eyy * 2.0, eyz * 2.0);

        // Posição do observador no cilindro
        let xd    = libm_sin(self.dalpha) * 0.9;
        let z_off = libm_sin(
            self.aa * 0.1 - self.thet * 0.2 + self.dalpha * 0.12001
        ) * 700.0;

        // Vetor de visada central (coordenadas esféricas → cartesianas)
        let cx = 4.0 * libm_cos(self.aa) * libm_cos(self.thet);
        let cy = 4.0 * libm_sin(self.aa) * libm_cos(self.thet);
        let cz = 4.0 * libm_sin(self.thet);

        // Rotação do plano de tela por eaa
        let ce = libm_cos(self.eaa);
        let se = libm_sin(self.eaa);

        let vx1 = ce * exx + se * eyx;
        let vy1 = ce * exy + se * eyy;
        let vz1 = ce * exz + se * eyz;

        let vx2 = -se * exx + ce * eyx;
        let vy2 = -se * exy + ce * eyy;
        let vz2 = -se * exz + ce * eyz;

        // Avança os ângulos (mesmos incrementos do original)
        self.aa     += 0.004;
        self.thet   += 0.006203;
        self.eaa    += 0.002;
        self.dalpha += 0.01;

        // Limpa o buffer antes de renderizar — evita lixo nas bordas
        // caso w/h não sejam múltiplos exatos de 8.
        renderer.clear_black();

        // --- Pré-cálculo e renderização ---
        self.do_precalc(w, h, xd, z_off, self.dalpha, cx, cy, cz, vx1, vy1, vz1, vx2, vy2, vz2);
        self.refresh(renderer, w, h);
    }
}

// -----------------------------------------------------------------------
// Funções matemáticas sem std — usa libm em bare metal
// -----------------------------------------------------------------------
//
// Adicione ao Cargo.toml:
//   [dependencies]
//   libm = "0.2"
//
// Se já estiver usando libm no projeto, basta importar aqui.

#[inline] fn libm_sin(x: f32)  -> f32 { libm::sinf(x) }
#[inline] fn libm_cos(x: f32)  -> f32 { libm::cosf(x) }
#[inline] fn libm_sqrt(x: f32) -> f32 { libm::sqrtf(x) }
#[inline] fn libm_atan2(y: f32, x: f32) -> f32 { libm::atan2f(y, x) }