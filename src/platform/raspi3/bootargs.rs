// src/platform/raspi3/bootargs.rs
//
// Único ponto do sistema que conhece tanto DemoKind quanto DiagKind.
// Mapeia a string de bootargs para um BootTarget.

use core::sync::atomic::{AtomicPtr, Ordering};
use crate::boot::boot_info::BootConfig;
use crate::demos::DemoKind;
use crate::diagnostics::DiagKind;

// Nomes dos ADFs passados na cmdline (df0=, df1=).
// Guardam ponteiros para slices dentro da cmdline (lifetime 'static via DTB).
static DF0_PTR: AtomicPtr<u8> = AtomicPtr::new(core::ptr::null_mut());
static DF0_LEN: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);
static DF1_PTR: AtomicPtr<u8> = AtomicPtr::new(core::ptr::null_mut());
static DF1_LEN: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);

fn store_disk(ptr: &AtomicPtr<u8>, len: &core::sync::atomic::AtomicUsize, s: &'static str) {
    ptr.store(s.as_ptr() as *mut u8, Ordering::Relaxed);
    len.store(s.len(), Ordering::Relaxed);
}

pub fn df0() -> Option<&'static str> {
    let p = DF0_PTR.load(Ordering::Relaxed);
    let l = DF0_LEN.load(Ordering::Relaxed);
    if p.is_null() || l == 0 { return None; }
    // SAFETY: pointer e len vêm da cmdline 'static
    Some(unsafe { core::str::from_utf8_unchecked(core::slice::from_raw_parts(p, l)) })
}

pub fn df1() -> Option<&'static str> {
    let p = DF1_PTR.load(Ordering::Relaxed);
    let l = DF1_LEN.load(Ordering::Relaxed);
    if p.is_null() || l == 0 { return None; }
    Some(unsafe { core::str::from_utf8_unchecked(core::slice::from_raw_parts(p, l)) })
}

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
                    "omega"          => BootTarget::Demo(DemoKind::Omega),

                    _ => return,
                };
            }

            "df0" => {
                // SAFETY: value é uma fatia da cmdline que vive em memória 'static (DTB)
                store_disk(&DF0_PTR, &DF0_LEN, unsafe {
                    core::mem::transmute::<&str, &'static str>(value)
                });
            }

            "df1" => {
                store_disk(&DF1_PTR, &DF1_LEN, unsafe {
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