use alloc::sync::{Arc, Weak};
use spin::{Mutex, Once};
use hypercraft::{VCpu, VmCpuMode, VmExitInfo};
use crate::hv::HyperCraftHalImpl;
use crate::hv::vm::VirtMach;
use crate::utils::CpuSet;

const MAX_VCPUS: usize = 4;
const BSP_CPU_ID: usize = 0;

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
    // arch specific vcpu
    inner_vcpu: VCpu<HyperCraftHalImpl>,
    vm: Weak<VirtMach>,
}


impl VirtCpu {
    pub fn is_bsp(&self) -> bool {
        self.vcpu_id == BSP_CPU_ID
    }
    pub fn vcpu_mode(&self) -> VmCpuMode {
        self.inner_vcpu.cpu_mode
    }
    pub fn phy_cpu_id(&self) {
        todo!()
    }
    pub fn set_start_up_entry(&mut self) { todo!() }
    pub fn is_paging_enabled(&self) -> bool { todo!() }
    pub fn reset(&mut self) {}
    pub fn launch(&self) {}
    pub fn kick(&self) {}
    pub fn prepare(&self) {}
    pub fn run(&mut self) -> Option<VmExitInfo> {
        self.inner_vcpu.run()
    }
}


pub struct VirtCpuSet {
    inner: [Once<Arc<Mutex<VirtCpu>>>; MAX_VCPUS],
}
