// src/platform/raspi3/bootargs.rs
//
// Único ponto do sistema que conhece tanto DemoKind quanto DiagKind.
// Mapeia a string de bootargs para um BootTarget.

use crate::boot::boot_info::BootConfig;
use crate::demos::DemoKind;
use crate::diagnostics::DiagKind;

#[derive(Clone, Copy, Debug)]
pub enum BootTarget {
    Diag(DiagKind),
    Demo(DemoKind),
}

pub fn apply_bootargs(args: &str, config: &mut BootConfig, target: &mut BootTarget) {
    for token in args.split_ascii_whitespace() {
        let Some((key, value)) = token.split_once('=') else {
            continue;
        };

        match key {
            "demo" => {
                *target = match value {
                    // diagnósticos
                    "gradient" => BootTarget::Diag(DiagKind::Gradient),
                    "testpattern" => BootTarget::Diag(DiagKind::TestPattern),
                    "smpte" => BootTarget::Diag(DiagKind::Smpte),

                    // demos
                    "rasterbars" => BootTarget::Demo(DemoKind::RasterBars),
                    "plasma" => BootTarget::Demo(DemoKind::Plasma),
                    "flame" => BootTarget::Demo(DemoKind::Flame),
                    "starfield" => BootTarget::Demo(DemoKind::Starfield),
                    "tunnel" => BootTarget::Demo(DemoKind::Tunnel),
                    "parallax" => BootTarget::Demo(DemoKind::Parallax),
                    "juggler" => BootTarget::Demo(DemoKind::Juggler),
                    "sprite_bouncer" => BootTarget::Demo(DemoKind::SpriteBouncer),

                    _ => return,
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

            _ => {}
        }
    }
}

fn parse_u32(s: &str) -> Option<u32> {
    if s.is_empty() {
        return None;
    }

    let mut result: u32 = 0;

    for b in s.bytes() {
        if !b.is_ascii_digit() {
            return None;
        }

        result = result
            .checked_mul(10)?
            .checked_add((b - b'0') as u32)?;
    }

    Some(result)
}