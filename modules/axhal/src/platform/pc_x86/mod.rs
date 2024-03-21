mod apic;
#[cfg(not(feature = "type1_5"))]
#[path = "boot.rs"]
mod boot;
#[cfg(feature = "type1_5")]
#[path = "boot_type15.rs"]
mod boot;
mod dtables;
mod uart16550;

pub(crate) use dtables::{kernel_stack_top, set_kernel_stack_top};

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
    #[cfg(not(feature = "type1_5"))]
    fn rust_main(cpu_id: usize, dtb: usize) -> !;
    #[cfg(feature = "type1_5")]
    fn rust_main(cpu_id: u32, linux_sp: usize) -> i32;
    #[cfg(feature = "smp")]
    fn rust_main_secondary(cpu_id: usize) -> !;
}

pub fn current_cpu_id() -> usize {
    match raw_cpuid::CpuId::new().get_feature_info() {
        Some(finfo) => finfo.initial_local_apic_id() as usize,
        None => 0,
    }
}

#[cfg(not(feature = "type1_5"))]
unsafe extern "C" fn rust_entry(magic: usize, _mbi: usize) {
    // TODO: handle multiboot info
    if magic == self::boot::MULTIBOOT_BOOTLOADER_MAGIC {
        crate::mem::clear_bss();
        crate::cpu::init_primary(current_cpu_id());
        self::uart16550::init();
        self::dtables::init_primary();
        self::time::init_early();
        rust_main(current_cpu_id(), 0);
    }
}

#[cfg(not(feature = "type1_5"))]
#[allow(unused_variables)]
unsafe extern "C" fn rust_entry_secondary(magic: usize) {
    #[cfg(feature = "smp")]
    if magic == self::boot::MULTIBOOT_BOOTLOADER_MAGIC {
        crate::cpu::init_secondary(current_cpu_id());
        self::dtables::init_secondary();
        rust_main_secondary(current_cpu_id());
    }
}

/// Initializes the platform devices for the primary CPU.
#[cfg(not(feature = "type1_5"))]
pub fn platform_init() {
    self::apic::init_primary();
    self::time::init_primary();
}

/// Initializes the platform devices for the primary CPU.
#[cfg(feature = "type1_5")]
pub fn platform_init() {
    self::dtables::init_primary();
    self::apic::init_primary();
    // self::time::init_primary();
}
/// Initializes the platform devices for secondary CPUs.
#[cfg(all(feature = "type1_5", feature = "smp"))]
pub fn platform_init_secondary() {
    self::dtables::init_secondary();
    // self::apic::init_secondary();
    // self::time::init_primary();
}

/// Initializes the platform devices for secondary CPUs.
#[cfg(all(not(feature = "type1_5"), feature = "smp"))]
pub fn platform_init_secondary() {
    self::apic::init_secondary();
    self::time::init_secondary();
}

#[cfg(feature = "type1_5")]
use core::sync::atomic::{AtomicU32, Ordering};
#[cfg(feature = "type1_5")]
static INIT_EARLY_OK: AtomicU32 = AtomicU32::new(0);
#[cfg(feature = "type1_5")]
// hypervisor start
extern "sysv64" fn rust_entry_hv(cpu_id: u32, linux_sp: usize) -> i32 {
    axlog::ax_println!("enter rust entry hv!!!");
    if cpu_id == 0 {
        primary_init_early(cpu_id);
    } else {
        while INIT_EARLY_OK.load(Ordering::Acquire) < 1 {
            core::hint::spin_loop();
        }
        crate::cpu::init_secondary(cpu_id as _);
    }
    let ret = unsafe { rust_main(cpu_id, linux_sp) };
    ret
}
#[cfg(feature = "type1_5")]
fn primary_init_early(cpu_id: u32) {
    // crate::mem::clear_bss();
    crate::cpu::init_primary(cpu_id as usize);
    self::uart16550::init();
    self::time::init_early();
    axlog::ax_println!("primary_init_early OK!!!");
    INIT_EARLY_OK.store(1, Ordering::Release);
}