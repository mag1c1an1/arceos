use crate::{Result, VCpu, HyperCraftHal};
// use axhal::hv::HyperCraftHalImpl;

pub const HVC_SHADOW_PROCESS_INIT: usize = 0x53686477;
pub const HVC_SHADOW_PROCESS_PRCS: usize = 0x70726373;
pub const HVC_SHADOW_PROCESS_RDY: usize = 0x52647921;

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
        _ => {
            warn!("Unhandled hypercall {}. vcpu: {:#x?}", id, vcpu);
        }
    }
    // Ok(0)
    // to compatible with jailhouse hypervisor test
    Ok(args.0)
    // Err(HyperError::NotSupported)
}
