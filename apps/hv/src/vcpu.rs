use alloc::sync::Arc;
use spin::Once;
use libax::hv::{HyperCraftHalImpl, VCpu};
use libax::sync::Mutex;
use crate::utils::CpuSet;

const MAX_VCPUS: usize = 4;

/// virtual cpu state
#[derive(Debug)]
pub enum VirtCpuState {
    Runnable,
    Running,
    Init,
    Block,
}


#[derive(Debug)]
pub struct VirtCpu {
    vcpu_id: usize,
    // vcpu only run on these cpus
    phy_cpu_affinity: CpuSet,
    vcpu_state: VirtCpuState,
    nr_sipi: u32,
    // arch specific vcpu
    arch_vcpu: VCpu<HyperCraftHalImpl>,
}


pub struct VirtCpuSet {
    inner: [Once<Arc<Mutex<VirtCpu>>>; MAX_VCPUS],
}
