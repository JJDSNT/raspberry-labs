// src/arch/aarch64/regs/mod.rs

pub mod cache;
pub mod system;
pub mod timer;

pub use cache::*;
pub use system::{
    CurrentEl, Daif, Elr, Esr, Far, Mair, Sctlr, Spsr, Tcr, Ttbr0, Vbar, VbarEl2,
};
pub use timer::{CntFrq, CntPct, CntpCtl, CntpTval};