use alloc::sync::Arc;
use spin::Once;
use axconfig::SMP;
use hypercraft::PerCpu;
use spinlock::SpinNoIrq;
use crate::hv::HyperCraftHalImpl;

pub static PHY_CPU_SET: Once<SpinNoIrq<PhyCpuSet>> = Once::new();


/// physical cpu
pub struct PhyCpu {
    per_cpu: PerCpu<HyperCraftHalImpl>,
    // current_vcpu: Option<Arc<Mutex<VirtCpu>>>,
}

impl PhyCpu {
    // pub fn new(cpu_id:usize) -> Self {
    //
    // }
    // pub fn cpu_id() -> usize {}
}


pub struct PhyCpuSet {
    inner: [Once<PhyCpu>; SMP],
}

impl PhyCpuSet {}