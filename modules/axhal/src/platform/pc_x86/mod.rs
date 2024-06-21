use crate::cpu;
use crate::cpu::this_cpu_id;

mod apic;
mod boot;
mod dtables;
mod uart16550;

pub mod mem;
pub mod misc;
pub mod time;

#[cfg(feature = "smp")]
pub mod mp;

#[cfg(feature = "irq")]
pub mod irq {
    pub use super::apic::*;
}

pub mod console {
    pub use super::uart16550::*;
}

extern "C" {
    fn rust_main(cpu_id: usize, dtb: usize) -> !;
    #[cfg(feature = "smp")]
    fn rust_main_secondary(cpu_id: usize) -> !;
}

fn current_cpu_id() -> usize {
    match raw_cpuid::CpuId::new().get_feature_info() {
        Some(finfo) => finfo.initial_local_apic_id() as usize,
        None => 0,
    }
}

unsafe extern "C" fn rust_entry(magic: usize, _mbi: usize) {
    // TODO: handle multiboot info
    if magic == boot::MULTIBOOT_BOOTLOADER_MAGIC {
        crate::mem::clear_bss();
        cpu::init_primary(current_cpu_id());
        uart16550::init();
        dtables::init_primary();
        time::init_early();
        rust_main(this_cpu_id(), 0);
    }
}

#[allow(unused_variables)]
unsafe extern "C" fn rust_entry_secondary(magic: usize) {
    #[cfg(feature = "smp")]
    if magic == boot::MULTIBOOT_BOOTLOADER_MAGIC {
        cpu::init_secondary(current_cpu_id());
        dtables::init_secondary();
        rust_main_secondary(this_cpu_id());
    }
}

/// Initializes the platform devices for the primary CPU.
pub fn platform_init() {
    apic::init_primary();
    time::init_primary();
}

/// Initializes the platform devices for secondary CPUs.
#[cfg(feature = "smp")]
pub fn platform_init_secondary() {
    apic::init_secondary();
    time::init_secondary();
}
