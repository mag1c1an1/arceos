//! Description tables (per-CPU GDT, per-CPU ISS, IDT)
use super::current_cpu_id;
use crate::arch::{GdtStruct, IdtStruct, TaskStateSegment};
use crate::mem::VirtAddr;
use lazy_init::LazyInit;
#[cfg(feature = "type1_5")]
use x86::{segmentation, segmentation::SegmentSelector};

use memoffset::offset_of;

static IDT: LazyInit<IdtStruct> = LazyInit::new();

#[percpu::def_percpu]
static TSS: LazyInit<TaskStateSegment> = LazyInit::new();

#[percpu::def_percpu]
static GDT: LazyInit<GdtStruct> = LazyInit::new();

#[allow(dead_code)]
pub const TSS_KERNEL_RSP_OFFSET: usize = offset_of!(TaskStateSegment, privilege_stack_table);

#[cfg(not(feature = "type1_5"))]
fn init_percpu() {
    unsafe {
        IDT.load();
        let tss = TSS.current_ref_mut_raw();
        let gdt = GDT.current_ref_mut_raw();
        tss.init_by(TaskStateSegment::new());
        gdt.init_by(GdtStruct::new(tss));
        gdt.load();
        gdt.load_tss();
        crate::arch::syscall::init_percpu();
    }
}

#[cfg(feature = "type1_5")]
fn init_percpu() {
    unsafe {
        debug!("CPU{} init_percpu", current_cpu_id());
        let tss = TSS.current_ref_mut_raw();
        let gdt = GDT.current_ref_mut_raw();
        tss.init_by(TaskStateSegment::new());
        gdt.init_by(GdtStruct::new(tss));
        gdt.load();
        segmentation::load_es(SegmentSelector::from_raw(0));
        // segmentation::load_cs(SegmentSelector::from_raw((GdtStruct::KCODE64_SELECTOR).0));
        segmentation::load_ss(SegmentSelector::from_raw(0));
        segmentation::load_ds(SegmentSelector::from_raw(0));
        IDT.load();
        gdt.load_tss();

        // PAT0: WB, PAT1: WC, PAT2: UC
        x86::msr::wrmsr(x86::msr::IA32_PAT, 0x070106);
        debug!("CPU{} finish init percpu", current_cpu_id());
    }
}

/// Initializes IDT, GDT on the primary CPU.
pub(super) fn init_primary() {
    debug!("\nInitialize IDT & GDT...");
    axlog::ax_println!("\nInitialize IDT & GDT...");
    IDT.init_by(IdtStruct::new());
    init_percpu();
}

/// Initializes IDT, GDT on secondary CPUs.
#[cfg(feature = "smp")]
pub(super) fn init_secondary() {
    init_percpu();
}

pub fn kernel_stack_top() -> VirtAddr {
    unsafe {
        let tss = TSS.current_ref_mut_raw();
        VirtAddr::from(tss.privilege_stack_table[0].as_u64() as usize)
    }
}

#[cfg(feature = "monolithic")]
pub fn set_kernel_stack_top(kstack_top: VirtAddr) {
    trace!("set percpu kernel stack: {:#x?}", kstack_top);
    unsafe {
        let tss = TSS.current_ref_mut_raw();
        tss.privilege_stack_table[0] = x86_64::VirtAddr::new(kstack_top.as_usize() as u64);

        crate::arch::syscall::set_kernel_stack(kstack_top.as_usize())
    }
}
