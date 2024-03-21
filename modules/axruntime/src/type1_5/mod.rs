#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(dead_code))]
#![feature(asm_sym)]
#![feature(asm_const)]
#![feature(lang_items)]
#![feature(concat_idents)]
#![feature(naked_functions)]
#![allow(unaligned_references)]

use alloc::string::String;
// use axlog::ax_println as println;
use core::sync::atomic::{AtomicI32, AtomicU32, Ordering};

mod consts;
mod config;
mod header;
mod memory;

pub use consts::HV_BASE;
pub use config::{HvSystemConfig, CellConfig, MemFlags};
pub use header::HvHeader;
pub use memory::{init_type15_allocator, activate_hv_pt, init_hv_page_table};
