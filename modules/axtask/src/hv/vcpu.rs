use alloc::string::{String, ToString};
use alloc::sync::{Arc, Weak};
use alloc::vec;
use core::cell::UnsafeCell;
use core::fmt::{Display, Formatter};
use core::time::Duration;
use spin::{Mutex, Once};
use axhal::cpu::this_cpu_id;
use axhal::time::busy_wait;
use hypercraft::{GuestPhysAddr, HostPhysAddr, HyperError, HyperResult, VCpu, VmCpuMode, VmExitInfo, VmxExitReason};
use crate::hv::HyperCraftHalImpl;
use crate::hv::notify::{hv_msg_handler, Message, send_message, Signal, wait_on_reply};
use crate::hv::prelude::vmcs_revision_id;
use crate::hv::vm::config::BSP_CPU_ID;
use crate::hv::vm::VirtMach;
use crate::hv::vmx::{handle_external_interrupt, handle_msr_read, handle_msr_write, X64VirtDevices};
use crate::on_timer_tick;
use crate::run_queue::RUN_QUEUE;
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
    pub vcpu_state: Mutex<VirtCpuState>,
    pub inner_vcpu: VCpu<HyperCraftHalImpl>,
    pub entry: Option<usize>,
    pub vm: Weak<Mutex<VirtMach>>,
    pub ept_root: HostPhysAddr,
    pub vm_name: String,
    pub x64_devices: X64VirtDevices,
    pub prev_pcpu: Option<usize>,
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

    pub fn vmcs_addr(&self) -> usize {
        self.get_inner().inner_vcpu.vmcs.phys_addr()
    }

    pub fn cpu_affinity(&self) -> CpuSet {
        self.get_inner().cpu_affinity.clone()
    }


    /// pre: Call this function only in switch context
    pub fn prepare(&self) -> HyperResult {
        // todo change is_launched
        match self.prev_pcpu() {
            None => {
                self.unbind_curr_cpu()?
            }
            Some(prev) => {
                if prev != this_cpu_id() {
                    let msg = Message::new(this_cpu_id(), prev, Signal::Clear, vec![self.vmcs_addr()]);
                    let reply = Message::new_reply(&msg);
                    loop {
                        send_message(msg.clone());
                        error!("{} send nmi to {}",this_cpu_id(),prev);
                        axhal::irq::send_nmi_to(prev);
                        error!("{} begin busy wait",self);
                        if wait_on_reply(&reply) {
                            break;
                        } else {
                            busy_wait(Duration::from_millis(100));
                        }
                    }
                    error!("{} finish busy wait",self);
                    // loop {
                    //     let msg = Message::new(this_cpu_id(), prev, Signal::Clear, vec![self.vmcs_addr()]);
                    //     let reply = Message::new_reply(&msg);
                    //     send_message(msg);
                    //     error!("{} send nmi to {}",this_cpu_id(),prev);
                    //     axhal::irq::send_nmi_to(prev);
                    //     error!("{} begin busy wait",self);
                    //     if wait_on_reply(reply) {
                    //         break;
                    //     } else {
                    //         busy_wait(Duration::from_millis(100));
                    //     }
                    // }
                    // error!("{} finish busy wait",self);
                    self.set_launched(false);
                }
            }
        }
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
                    vcpu_state: Mutex::new(VirtCpuState::Init),
                    inner_vcpu: VCpu::new_common(vmcs_revision_id(), 0)?,
                    entry: Some(entry),
                    vm: weak,
                    ept_root,
                    vm_name,
                    x64_devices: X64VirtDevices::new()?,
                    prev_pcpu: None,
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
                    vcpu_state: Mutex::new(VirtCpuState::Init),
                    inner_vcpu: VCpu::new_common(vmcs_revision_id(), vcpu_id)?,
                    entry: None,
                    vm: weak,
                    ept_root,
                    vm_name,
                    x64_devices: X64VirtDevices::new()?,
                    prev_pcpu: None,
                })
            }
        ))
    }

    pub fn set_prev_pcpu(&self, cpu_id: usize) {
        self.get_inner_mut().prev_pcpu = Some(cpu_id);
    }

    pub fn prev_pcpu(&self) -> Option<usize> {
        self.get_inner().prev_pcpu
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
        *self.get_inner().vcpu_state.lock()
    }
    pub fn set_state(&self, state: VirtCpuState) {
        *self.get_inner_mut().vcpu_state.lock() = state;
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
    pub fn start(&self) {
        while self.state() != VirtCpuState::Offline {
            // error!("{} exec",self);
            match self.run() {
                None => {}
                Some(_) => {
                    match self.vmexit_handler() {
                        Ok(_) => { continue }
                        Err(e) => {
                            if matches!(e, HyperError::Shutdown) {
                                debug!("shutdown");
                                self.vm().unwrap().lock().shutdown();
                                break;
                            } else {
                                panic!("hyper error{:?}", e);
                            }
                        }
                    }
                }
            }
        }
        error!("{} shutdown",self);
    }

    fn vmexit_handler(&self) -> HyperResult {
        let vmx_vcpu = self.vmx_vcpu_mut();
        let exit_info = vmx_vcpu.exit_info()?;

        match exit_info.exit_reason {
            VmxExitReason::EXTERNAL_INTERRUPT => handle_external_interrupt(self),
            VmxExitReason::IO_INSTRUCTION => self.get_inner_mut().x64_devices.handle_io_instruction(vmx_vcpu, &exit_info),
            VmxExitReason::MSR_READ => handle_msr_read(self),
            VmxExitReason::MSR_WRITE => handle_msr_write(self),
            VmxExitReason::PREEMPTION_TIMER => self.handle_vmx_preemption_timer(),
            VmxExitReason::SIPI => todo!("todo sipi"),
            VmxExitReason::EXCEPTION_NMI => {
                // panic!("vm nmi exit");
                hv_msg_handler(this_cpu_id());
                Ok(())
            }
            // VmxExitReason::EPT_VIOLATION => ,
            _ => panic!(
                "[{}] vmexit reason not supported {:?}:\n",
                self.vcpu_id(),
                exit_info.exit_reason
            ),
        }
    }

    fn handle_vmx_preemption_timer(&self) -> HyperResult {
        // error!("vmx preemption timer");
        // RUN_QUEUE.lock().hv_scheduler_timer_tick();
        on_timer_tick();
        self.reset_vmx_preemption_timer()
    }
}
