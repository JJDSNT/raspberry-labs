// src/platform/bootargs.rs
//
// Mapeia a string de bootargs (ex: "demo=rasterbars width=1280 height=720")
// para um BootConfig, sem alloc.

use crate::demos::DemoKind;
use crate::BootConfig;

/// Parseia `bootargs` e aplica os valores encontrados sobre `config`.
/// Campos não presentes nos bootargs mantêm o valor default.
/// Suporta múltiplos espaços/tabs entre tokens (split_ascii_whitespace).
pub fn apply_bootargs(args: &str, config: &mut BootConfig) {
    for token in args.split_ascii_whitespace() {
        let Some((key, value)) = token.split_once('=') else {
            continue;
        };

        match key {
            "demo" => {
                config.demo = match value {
                    "gradient" => DemoKind::Gradient,
                    "testpattern" => DemoKind::TestPattern,
                    "rasterbars" => DemoKind::RasterBars,
                    "plasma" => DemoKind::Plasma,
                    "flame" => DemoKind::Flame,
                    "starfield" => DemoKind::Starfield,
                    _ => config.demo, // valor desconhecido: mantém o atual
                };
            }
            "width" => {
                if let Some(n) = parse_u32(value) {
                    config.width = n;
                }
            }
            "height" => {
                if let Some(n) = parse_u32(value) {
                    config.height = n;
                }
            }
            "depth" => {
                if let Some(n) = parse_u32(value) {
                    config.depth = n;
                }
            }
            _ => {} // chave desconhecida, ignora silenciosamente
        }
    }
}

/// Parseia um `&str` ASCII de dígitos decimais como `u32`.
/// Retorna `None` para strings vazias, não-numéricas ou overflow.
fn parse_u32(s: &str) -> Option<u32> {
    if s.is_empty() {
        return None;
    }

    let mut result: u32 = 0;
    for b in s.bytes() {
        if !b.is_ascii_digit() {
            return None;
        }

        result = result.checked_mul(10)?.checked_add((b - b'0') as u32)?;
    }

    Some(result)
}