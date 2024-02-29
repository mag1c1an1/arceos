use crate::Result;
use axhal::hv::HyperCraftHalImpl;
pub use hypercraft::VCpu;
use hypercraft::HostPhysAddr;
#[cfg(feature = "type1_5")]
use hypercraft::LinuxContext;

#[cfg(feature = "type1_5")]
pub fn new_vcpu(
    vcpu_id: usize,
    vmcs_revision_id: u32,
    ept_root: HostPhysAddr,
    linux: &LinuxContext,
) -> Result<VCpu<HyperCraftHalImpl>> {
    let vcpu = VCpu::<HyperCraftHalImpl>::new_type15(vcpu_id, vmcs_revision_id, ept_root, linux)?;
    Ok(vcpu)
}
