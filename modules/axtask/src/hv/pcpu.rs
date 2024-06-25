use core::cell::UnsafeCell;
use lazy_static::lazy_static;
use spin::Once;
use axconfig::SMP;
use axhal::cpu::this_cpu_id;
use hypercraft::PerCpu;
use crate::hv::HyperCraftHalImpl;

lazy_static! {
    static ref PHY_CPU_SET: PhyCpuSet = PhyCpuSet::new();
}

/// physical cpu
pub struct PhyCpu {
    per_cpu: PerCpu<HyperCraftHalImpl>,
}

impl PhyCpu {
    pub fn new() -> Self {
        Self {
            per_cpu: PerCpu::new(this_cpu_id()),
        }
    }
    pub fn cpu_id() -> usize {
        this_cpu_id()
    }
    pub fn enable_vmx(&mut self) {
        self.per_cpu.hardware_enable().unwrap()
    }

    pub fn disable_vmx(&mut self) {
        self.per_cpu.hardware_disable().unwrap()
    }
}


const ARRAY_REPEAT_VALUE: Once<PhyCpu> = Once::new();

#[derive(Debug)]
pub struct PhyCpuSet {
    inner: UnsafeCell<[Once<PhyCpu>; SMP]>,
}

impl PhyCpuSet {
    pub fn new() -> Self {
        PhyCpuSet { inner: UnsafeCell::new([ARRAY_REPEAT_VALUE; SMP]) }
    }

    pub fn init(&self, cpu_id: usize) {
        let inner = self.get_inner_mut();
        inner[cpu_id].call_once(|| PhyCpu::new());
    }

    pub fn get_phy_cpu_mut(&self, cpu_id: usize) -> &mut PhyCpu {
        self.get_inner_mut()[cpu_id].get_mut().unwrap()
    }

    fn get_inner(&self) -> &[Once<PhyCpu>; SMP] {
        unsafe {
            &*self.inner.get()
        }
    }

    fn get_inner_mut(&self) -> &mut [Once<PhyCpu>; SMP] {
        unsafe {
            &mut *self.inner.get()
        }
    }

    /// pre: bsp is initialized
    pub fn vmcs_revision_id(&self) -> u32 {
        self.get_phy_cpu_mut(0).per_cpu.vmcs_revision_id()
    }
}

unsafe impl Send for PhyCpuSet {}

unsafe impl Sync for PhyCpuSet {}


pub fn curr_phy_cpu() -> &'static mut PhyCpu {
    PHY_CPU_SET.get_phy_cpu_mut(this_cpu_id())
}

/// init phy cpu
/// enable vmx
pub fn phy_cpu_init() {
    PHY_CPU_SET.init(this_cpu_id());
    curr_phy_cpu().enable_vmx();
}

/// all phy cpus' vmcs_revision_id is same
pub fn vmcs_revision_id() -> u32 {
    PHY_CPU_SET.vmcs_revision_id()
}