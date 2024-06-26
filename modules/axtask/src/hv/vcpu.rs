use alloc::string::{String, ToString};
use alloc::sync::{Arc, Weak};
use core::cell::UnsafeCell;
use core::fmt::{Display, Formatter};
use spin::{Mutex, Once};
use hypercraft::{GuestPhysAddr, HostPhysAddr, HyperResult, VCpu, VmCpuMode, VmExitInfo};
use crate::hv::HyperCraftHalImpl;
use crate::hv::prelude::vmcs_revision_id;
use crate::hv::vm::config::BSP_CPU_ID;
use crate::hv::vm::VirtMach;
use crate::utils::CpuSet;


/// virtual cpu state
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
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
    pub entry: Option<usize>,
    pub vm: Weak<Mutex<VirtMach>>,
    pub ept_root: HostPhysAddr,
    pub vm_name: String,
}


#[derive(Debug)]
pub struct VirtCpu {
    inner: UnsafeCell<VirCpuInner>,
}

impl Display for VirtCpu {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{}[{}]", self.get_inner().vm_name, self.vcpu_id()))
    }
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

    /// pre: Call this function only in switch context
    pub fn prepare(&self) -> HyperResult {
        // todo change is_launched
        self.bind_curr_cpu()?;
        if self.state() == VirtCpuState::Init {
            error!("vcpu {} in prepare", self.vcpu_id());
            let inner_vcpu = &mut self.get_inner_mut().inner_vcpu;
            let entry = self.get_inner().entry;
            let ept_root = self.get_inner().ept_root;
            inner_vcpu.setup_vmcs(entry, ept_root)?;
            self.set_state(VirtCpuState::Running);
            self.set_launched(false);
        }
        Ok(())
    }

    /// create new bsp vcpu
    pub fn new_bsp(vm_name: String, cpu_affinity: CpuSet, weak: Weak<Mutex<VirtMach>>, entry: GuestPhysAddr, ept_root: HostPhysAddr) -> HyperResult<Arc<Self>> {
        Ok(Arc::new(
            Self {
                inner: UnsafeCell::new(VirCpuInner {
                    vcpu_id: 0,
                    cpu_affinity,
                    vcpu_state: VirtCpuState::Init,
                    inner_vcpu: VCpu::new_common(vmcs_revision_id(), 0)?,
                    entry: Some(entry),
                    vm: weak,
                    ept_root,
                    vm_name,
                })
            }
        ))
    }

    pub fn new_ap(vm_name: String, vcpu_id: usize, cpu_affinity: CpuSet, weak: Weak<Mutex<VirtMach>>, ept_root: HostPhysAddr) -> HyperResult<Arc<Self>> {
        Ok(Arc::new(
            Self {
                inner: UnsafeCell::new(VirCpuInner {
                    vcpu_id,
                    cpu_affinity,
                    vcpu_state: VirtCpuState::Init,
                    inner_vcpu: VCpu::new_common(vmcs_revision_id(), vcpu_id)?,
                    entry: None,
                    vm: weak,
                    ept_root,
                    vm_name,
                })
            }
        ))
    }

    pub fn is_bsp(&self) -> bool {
        self.get_inner().vcpu_id == BSP_CPU_ID
    }
    pub fn vcpu_mode(&self) -> VmCpuMode {
        self.get_inner().inner_vcpu.cpu_mode
    }
    pub fn set_start_up_entry(&self, entry: usize) {
        self.get_inner_mut().entry = Some(entry);
    }
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
    pub fn vcpu_id(&self) -> usize {
        self.get_inner().vcpu_id
    }
    pub fn vmx_vcpu_mut(&self) -> &mut VCpu<HyperCraftHalImpl> {
        &mut self.get_inner_mut().inner_vcpu
    }

    pub fn vmx_vcpu(&self) -> &VCpu<HyperCraftHalImpl> {
        &self.get_inner().inner_vcpu
    }

    pub fn set_sipi_num(&self, nr_sipi: u8) {
        self.get_inner_mut().inner_vcpu.nr_sipi = nr_sipi;
    }
    pub fn sipi_num(&self) -> u8 {
        self.get_inner_mut().inner_vcpu.nr_sipi
    }
    pub fn state(&self) -> VirtCpuState {
        self.get_inner().vcpu_state
    }
    pub fn set_state(&self, state: VirtCpuState) {
        self.get_inner_mut().vcpu_state = state;
    }

    pub fn vm(&self) -> Option<Arc<Mutex<VirtMach>>> {
        self.get_inner().vm.upgrade()
    }
    pub fn set_launched(&self, val: bool) {
        self.get_inner_mut().inner_vcpu.is_launched = val;
    }
    pub fn reset_vmx_preemption_timer(&self) -> HyperResult {
        self.get_inner_mut().inner_vcpu.reset_timer()
    }
}
