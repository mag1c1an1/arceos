use alloc::sync::Arc;
use spin::Once;
use libax::hv::{HyperCraftHalImpl, PerCpu};
use libax::prelude::num_cpus;
use libax::sync::Mutex;
use crate::vcpu::VirtCpu;

pub static PHY_CPU_SET: Once<Mutex<PhyCpuSet>> = Once::new();


/// physical cpu
pub struct PhyCpu {
    per_cpu: PerCpu<HyperCraftHalImpl>,
    current_vcpu: Option<Arc<Mutex<VirtCpu>>>,
}

impl PhyCpu {
   pub fn new(cpu_id:usize) -> Self {

   }
    pub fn cpu_id() -> usize {}
}


pub struct PhyCpuSet {
    inner: [Once<PhyCpu>; num_cpus()],
}

impl PhyCpuSet {}