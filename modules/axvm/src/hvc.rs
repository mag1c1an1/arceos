use crate::{Result, VCpu, HyperCraftHal};
use axhal::{current_cpu_id, mem::phys_to_virt};
use axhal::mem::PhysAddr;
// use axhal::hv::HyperCraftHalImpl;

pub const HVC_SHADOW_PROCESS_INIT: usize = 0x53686477;
pub const HVC_SHADOW_PROCESS_PRCS: usize = 0x70726373;
pub const HVC_SHADOW_PROCESS_RDY: usize = 0x52647921;
pub const HVC_AXTASK_UP: usize = 0x9;

pub fn handle_hvc<H: HyperCraftHal>(vcpu: &mut VCpu<H>, id: usize, args: (usize, usize, usize)) -> Result<u32> {
    info!(
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
        HVC_AXTASK_UP => {
            let gpm = crate::config::root_gpm();
            info!("{:#x?}", gpm);
            let phy_addr = gpm.translate(args.2)?;
            info!("{:#x?}", phy_addr);
            let code = unsafe { core::slice::from_raw_parts(phys_to_virt(PhysAddr::from(phy_addr)).as_ptr(), 110)};
            info!("content: {:?}: ", code);
            axtask_up(args.0);
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

fn axtask_up(cpuset: usize) {
    let current_cpu = current_cpu_id();
    let cpuset = cpuset as usize;
    let num_bits = core::mem::size_of::<u32>() * 8;
    for i in 0..num_bits {
        if cpuset & (1 << i) != 0 {
            info!("CPU{} send nmi ipi to CPU{} ", current_cpu, i);
            axhal::irq::send_nmi_to(i);
            // todo!();
        }
    }
}