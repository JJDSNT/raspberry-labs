// src/platform/raspi3/bootargs.rs
//
// Único ponto do sistema que conhece tanto DemoKind quanto DiagKind.
// Mapeia a string de bootargs para um BootTarget.

use core::sync::atomic::{AtomicPtr, Ordering};
use crate::boot::boot_info::BootConfig;
use crate::demos::DemoKind;
use crate::diagnostics::DiagKind;

// Nomes de ficheiros passados na cmdline — ponteiros para slices 'static do DTB.
use core::sync::atomic::AtomicUsize;

static DF0_PTR: AtomicPtr<u8>  = AtomicPtr::new(core::ptr::null_mut());
static DF0_LEN: AtomicUsize    = AtomicUsize::new(0);
static DF1_PTR: AtomicPtr<u8>  = AtomicPtr::new(core::ptr::null_mut());
static DF1_LEN: AtomicUsize    = AtomicUsize::new(0);
static ROM_PTR: AtomicPtr<u8>  = AtomicPtr::new(core::ptr::null_mut());
static ROM_LEN: AtomicUsize    = AtomicUsize::new(0);

fn store_str(ptr: &AtomicPtr<u8>, len: &AtomicUsize, s: &'static str) {
    ptr.store(s.as_ptr() as *mut u8, Ordering::Relaxed);
    len.store(s.len(), Ordering::Relaxed);
}

fn load_str(ptr: &AtomicPtr<u8>, len: &AtomicUsize) -> Option<&'static str> {
    let p = ptr.load(Ordering::Relaxed);
    let l = len.load(Ordering::Relaxed);
    if p.is_null() || l == 0 { return None; }
    Some(unsafe { core::str::from_utf8_unchecked(core::slice::from_raw_parts(p, l)) })
}

pub fn df0() -> Option<&'static str> { load_str(&DF0_PTR, &DF0_LEN) }
pub fn df1() -> Option<&'static str> { load_str(&DF1_PTR, &DF1_LEN) }
pub fn rom() -> Option<&'static str> { load_str(&ROM_PTR, &ROM_LEN) }

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
                    "gfx3d_triangle" => BootTarget::Demo(DemoKind::Gfx3dTriangle),
                    "omega"          => BootTarget::Demo(DemoKind::Omega),

                    _ => return,
                };
            }

            "df0" => {
                store_str(&DF0_PTR, &DF0_LEN, unsafe {
                    core::mem::transmute::<&str, &'static str>(value)
                });
            }
            "df1" => {
                store_str(&DF1_PTR, &DF1_LEN, unsafe {
                    core::mem::transmute::<&str, &'static str>(value)
                });
            }
            "rom" => {
                store_str(&ROM_PTR, &ROM_LEN, unsafe {
                    core::mem::transmute::<&str, &'static str>(value)
                });
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