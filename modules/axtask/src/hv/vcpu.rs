use alloc::sync::{Arc, Weak};
use core::cell::UnsafeCell;
use spin::{Mutex, Once};
use hypercraft::{GuestPhysAddr, HostPhysAddr, HyperResult, VCpu, VmCpuMode, VmExitInfo};
use crate::hv::HyperCraftHalImpl;
use crate::hv::prelude::vmcs_revision_id;
use crate::hv::vm::config::BSP_CPU_ID;
use crate::hv::vm::VirtMach;
use crate::utils::CpuSet;


/// virtual cpu state
#[derive(Debug)]
pub enum VirtCpuState {
    Init,
    Offline,
    Running,
    Stop,
}


struct VirCpuInner {
    pub vcpu_id: usize,
    // vcpu only run on these cpus
    pub cpu_affinity: CpuSet,
    pub vcpu_state: VirtCpuState,
    pub inner_vcpu: VCpu<HyperCraftHalImpl>,
    vm: Weak<Mutex<VirtMach>>,
}


#[derive(Debug)]
pub struct VirtCpu {
    inner: UnsafeCell<VirCpuInner>,
}


impl VirtCpu {
    fn get_inner(&self) -> &VirCpuInner {
        unsafe {
            &*self.inner.get()
        }
    }
    fn get_inner_mut(&self) -> &mut VirCpuInner {
        unsafe {
            &mut *self.inner.get()
        }
    }

    /// create new bsp vcpu
    /// unbind on any phy cpu
    pub fn new_bsp(cpu_affinity: CpuSet, weak: Weak<Mutex<VirtMach>>, entry: GuestPhysAddr, ept_root: HostPhysAddr) -> HyperResult<Arc<Self>> {
        let mut inner_vcpu = VCpu::new_common(vmcs_revision_id())?;
        {
            // bind to current cpu
            // setup vmcs
            // unbind
            inner_vcpu.bind_to_current_cpu()?;
            inner_vcpu.setup_vmcs(Some(entry), ept_root)?;
            inner_vcpu.unbind_to_current_cpu()?;
        }
        Ok(Arc::new(
            Self {
                inner: UnsafeCell::new(VirCpuInner {
                    vcpu_id: 0,
                    cpu_affinity,
                    vcpu_state: VirtCpuState::Init,
                    inner_vcpu,
                    vm: weak,
                })
            }
        ))
    }

    pub fn new_ap() -> Arc<Self> {
        todo!()
    }

    fn common_vcpu(vcpu_id: usize) -> Self {
        todo!()
    }

    pub fn is_bsp(&self) -> bool {
        self.get_inner().vcpu_id == BSP_CPU_ID
    }
    pub fn vcpu_mode(&self) -> VmCpuMode {
        self.get_inner().inner_vcpu.cpu_mode
    }
    pub fn set_start_up_entry(&self) { todo!() }
    pub fn reset(&mut self) {}
    pub fn bind_curr_cpu(&self) -> HyperResult {
        self.get_inner().inner_vcpu.bind_to_current_cpu()
    }
    pub fn unbind_curr_cpu(&self) -> HyperResult {
        self.get_inner().inner_vcpu.unbind_to_current_cpu()
    }
    pub fn run(&self) -> Option<VmExitInfo> {
        self.get_inner_mut().inner_vcpu.run()
    }
    pub fn offline(&self) {
        self.get_inner_mut().vcpu_state = VirtCpuState::Offline;
    }
    pub fn start(&self) {
        self.get_inner_mut().vcpu_state = VirtCpuState::Running
    }

    pub fn vmx_cpu_mut(&self) -> &mut VCpu<HyperCraftHalImpl> {
        &mut self.get_inner_mut().inner_vcpu
    }
}
