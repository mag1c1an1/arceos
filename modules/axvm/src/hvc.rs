use alloc::string::String;

use axalloc::GlobalPage;
use axhal::current_cpu_id;
use axhal::mem::{phys_to_virt, virt_to_phys, PhysAddr};
use memory_addr::PAGE_SIZE_4K;

use crate::config::entry::{vm_cfg_add_vm_entry, vm_cfg_entry, VMCfgEntry};
use crate::Error;
use crate::{nmi::nmi_send_msg, nmi::NmiMessage, HyperCraftHal, Result, VCpu};
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
    vm_id: usize,
    /// Reserved.
    vm_type: usize,
    /// VM cpu mask.
    cpu_mask: usize,
    /// Size of BIOS.
    bios_size: usize,
    /// Physical addr of BIOS, set by ArceOS hypervisor.
    bios_load_physical_addr: usize,
    /// Size of KERNEL.
    kernel_size: usize,
    /// Physical addr of kernel image, set by ArceOS hypervisor.
    kernel_load_physical_addr: usize,
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

            let _ = ax_hvc_create_vm(arg)?;
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
const fn align_up_4k(pos: usize) -> usize {
    (pos + PAGE_SIZE_4K - 1) & !(PAGE_SIZE_4K - 1)
}

fn ax_hvc_create_vm(cfg: &mut ArceosAxvmCreateArg) -> Result<u32> {
    let mut vm_cfg_entry = VMCfgEntry::new(
        String::from("Guest VM"),
        String::from("Guest cmdline"),
        crate::config::BIOS_ENTRY,
        cfg.cpu_mask,
    );

    let bios_img_size = align_up_4k(cfg.bios_size);
    let bios_loaded_pages =
        GlobalPage::alloc_contiguous(bios_img_size / PAGE_SIZE_4K, PAGE_SIZE_4K).map_err(|e| {
            warn!(
                "failed to allocate {} Bytes memory for bios, err {:?}",
                bios_img_size, e
            );
            Error::NoMemory
        })?;

    cfg.bios_load_physical_addr = bios_loaded_pages.start_paddr(virt_to_phys).as_usize();
    vm_cfg_entry.set_bios_loaded_pages(bios_loaded_pages);

    let kernel_img_size = align_up_4k(cfg.kernel_size);
    let kernel_loaded_pages =
        GlobalPage::alloc_contiguous(kernel_img_size / PAGE_SIZE_4K, PAGE_SIZE_4K).map_err(
            |e| {
                warn!(
                    "failed to allocate {} Bytes memory for kernel",
                    kernel_img_size
                );
                Error::NoMemory
            },
        )?;

    cfg.kernel_load_physical_addr = kernel_loaded_pages.start_paddr(virt_to_phys).as_usize();
    vm_cfg_entry.set_kernel_img_loaded_pages(kernel_loaded_pages);

    let vm_id = vm_cfg_add_vm_entry(vm_cfg_entry)?;
    cfg.vm_id = vm_id;

    Ok(vm_id as u32)
}

fn ax_hvc_boot_vm(vm_id: usize) {
    let vm_cfg_entry = match vm_cfg_entry(vm_id) {
        Some(entry) => entry,
        None => {
            warn!("VM {} not existed, boot vm failed", vm_id);
            return;
        }
    };
    let cpuset = vm_cfg_entry.get_cpu_set();

    info!("boot VM {} on cpuset {:#x}", vm_id, cpuset);

    let current_cpu = current_cpu_id();
    let num_bits = core::mem::size_of::<u32>() * 8;
    let msg = NmiMessage::BootVm(vm_id);
    for i in 0..num_bits {
        if cpuset & (1 << i) != 0 {
            info!("CPU{} send nmi ipi to CPU{} ", current_cpu, i);
            // axhal::irq::send_nmi_to(i);
            nmi_send_msg(i, msg);
            // todo!();
        }
    }
}
