// src/boot/boot_info.rs

use core::ptr::NonNull;

use crate::demos::DemoKind;
use crate::platform::raspi3::bootargs::BootTarget;

/// Configuração inicial do sistema (resolução, profundidade, etc)
#[derive(Clone, Copy, Debug)]
pub struct BootConfig {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

impl BootConfig {
    pub const fn default() -> Self {
        Self {
            width: 1024,
            height: 768,
            depth: 32,
        }
    }
}

/// Informações completas passadas do boot para o kernel
pub struct BootInfo {
    /// Ponteiro para o Device Tree Blob (DTB)
    pub dtb: Option<NonNull<u8>>,

    /// Linha de comando (bootargs)
    pub cmdline: Option<&'static str>,

    /// Configuração gráfica inicial
    pub config: BootConfig,

    /// O que executar após boot (demo/diag)
    pub target: BootTarget,

    /// Framebuffer inicial (se disponível futuramente)
    pub framebuffer: Option<FramebufferInfo>,
}

impl BootInfo {
    /// Cria estrutura inicial a partir do DTB
    pub fn default_with_dtb(dtb_ptr: usize) -> Self {
        Self {
            dtb: NonNull::new(dtb_ptr as *mut u8),
            cmdline: None,
            config: BootConfig::default(),
            target: BootTarget::Demo(DemoKind::Flame),
            framebuffer: None,
        }
    }
}

/// Informação de framebuffer (para futuro uso mais cedo no boot)
pub struct FramebufferInfo {
    pub addr: usize,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub bpp: u32,
}