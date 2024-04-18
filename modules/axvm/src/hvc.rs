use crate::{nmi::nmi_send_msg, nmi::NmiMessage, HyperCraftHal, Result, VCpu};
use axhal::mem::PhysAddr;
use axhal::{current_cpu_id, mem::phys_to_virt};
use axalloc::GlobalPage;
use memory_addr::PAGE_SIZE_4K;
// use axhal::hv::HyperCraftHalImpl;

pub const HVC_SHADOW_PROCESS_INIT: usize = 0x53686477;
pub const HVC_SHADOW_PROCESS_PRCS: usize = 0x70726373;
pub const HVC_SHADOW_PROCESS_RDY: usize = 0x52647921;

pub const HVC_AXVM_CREATE_CFG: usize = 0x101;
pub const HVC_AXVM_LOAD_IMG: usize = 0x102;
pub const HVC_AXVM_BOOT: usize = 0x103;

#[derive(Debug)]
#[repr(C, packed)]
struct ArceosAxvmCreateArg {
    /// VM ID, set by ArceOS hypervisor.
    vm_id: u64,
    /// Reserved.
    vm_type: u64,
    /// VM cpu mask.
    cpu_mask: u64,
    /// Size of BIOS.
    bios_size: u64,
    /// Physical addr of BIOS, set by ArceOS hypervisor.
    bios_load_physical_addr: u64,
    /// Size of KERNEL.
    kernel_size: u64,
    /// Physical addr of kernel image, set by ArceOS hypervisor.
    kernel_load_physical_addr: u64,
}

pub fn handle_hvc<H: HyperCraftHal>(
    vcpu: &mut VCpu<H>,
    id: usize,
    args: (usize, usize, usize),
) -> Result<u32> {
    debug!(
        "hypercall_handler vcpu: {}, id: {:#x?}, args: {:#x?}, {:#x?}, {:#x?}",
        vcpu.vcpu_id(),
        id,
        args.0,
        args.1,
        args.2
    );

    match id {
        HVC_SHADOW_PROCESS_INIT => {
            axtask::notify_all_process();
        }
        HVC_AXVM_CREATE_CFG => {
            // Translate guest physical address of ArceosAxvmCreateArg into virtual address of hypervisor.
            let arg_gpa = args.0;
            let arg_hpa = crate::config::root_gpm().translate(arg_gpa)?;
            let arg_hva = phys_to_virt(PhysAddr::from(arg_hpa)).as_mut_ptr();

            let arg = unsafe { &mut *{ arg_hva as *mut ArceosAxvmCreateArg } };

            debug!("HVC_AXVM_CREATE_CFG get\n{:#x?}", arg);

            ax_hvc_create_vm(arg);
        }
        HVC_AXVM_LOAD_IMG => {
            warn!("HVC_AXVM_LOAD_IMG is combined with HVC_AXVM_CREATE_CFG currently");
            warn!("Just return");
        }
        HVC_AXVM_BOOT => {
            ax_hvc_boot_vm(args.0);
        }
        _ => {
            warn!("Unhandled hypercall {}. vcpu: {:#x?}", id, vcpu);
        }
    }
    // Ok(0)
    // to compatible with jailhouse hypervisor test
    Ok(id as u32)
    // Err(HyperError::NotSupported)
}

#[inline]
const fn align_up(pos: usize, align: usize) -> usize {
    (pos + align - 1) & !(align - 1)
}

fn ax_hvc_create_vm(cfg: &mut ArceosAxvmCreateArg) {
    cfg.vm_id = super::vm::generate_vm_id() as u64;

    // let bios_loaded_addr = GlobalPage::alloc_contiguous(num_pages, PAGE_SIZE_4K);

    cfg.bios_load_physical_addr = 0xffff;
    cfg.kernel_load_physical_addr = 0xffff;
}

fn ax_hvc_boot_vm(vm_id: usize) {
    let cpuset = 0;
    info!("boot VM {} on cpuset {:#x}", vm_id, cpuset);
    return;
    let current_cpu = current_cpu_id();
    let num_bits = core::mem::size_of::<u32>() * 8;
    let msg = NmiMessage::NIMBOS(0x8000, 0);
    for i in 0..num_bits {
        if cpuset & (1 << i) != 0 {
            info!("CPU{} send nmi ipi to CPU{} ", current_cpu, i);
            // axhal::irq::send_nmi_to(i);
            nmi_send_msg(i, msg);
            // todo!();
        }
    }
}
