use crate::{Result, VCpu, HyperCraftHal};
use axhal::current_cpu_id;
// use axhal::hv::HyperCraftHalImpl;

pub const HVC_SHADOW_PROCESS_INIT: usize = 0x53686477;
pub const HVC_SHADOW_PROCESS_PRCS: usize = 0x70726373;
pub const HVC_SHADOW_PROCESS_RDY: usize = 0x52647921;
pub const HVC_AXPROCESS_UP: usize = 0x9;

pub fn handle_hvc<H: HyperCraftHal>(vcpu: &mut VCpu<H>, id: usize, args: (u32, u32)) -> Result<u32> {
    info!(
        "hypercall_handler vcpu: {}, id: {:#x?}, args: {:#x?}, {:#x?}",
        vcpu.vcpu_id(),
        id,
        args.0,
        args.1
    );

    match id {
        HVC_SHADOW_PROCESS_INIT => {
            axtask::notify_all_process();
        }
        HVC_AXPROCESS_UP => {
            axprocess_up(args.0);
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

fn axprocess_up(cpuset: u32) {
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