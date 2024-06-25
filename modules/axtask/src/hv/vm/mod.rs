//! Abstraction of a virtual machine

use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::{Debug, Display, Formatter, Write};
use core::ptr::addr_of;
use core::sync::atomic::{AtomicUsize, Ordering};
use hashbrown::HashMap;
use lazy_static::lazy_static;
use spin::{Mutex, Once};
use axhal::mem::{MemRegion, virt_to_phys, VirtAddr};
use crate::hv::mm::{GuestMemoryRegion, GuestPhysMemorySet, load_guest_image};

pub mod config;

pub use config::VmConfig;
pub use config::arceos_config;
use hypercraft::{GuestPhysAddr, HostPhysAddr, HostVirtAddr, HyperError, HyperResult, PerCpu, VCpu, VmxExitInfo};
use page_table_entry::MappingFlags;
use spinlock::SpinNoIrq;
use crate::hv::{HyperCraftHalImpl, vmx};
use crate::hv::vcpu::VirtCpu;
use crate::hv::vm::config::BSP_CPU_ID;
use crate::{AxTaskRef, spawn_vcpu};
use crate::utils::CpuSet;


/// global virtual machine hashmap
static mut VM_TABLE: Once<HashMap<usize, Arc<Mutex<VirtMach>>>> = Once::new();

pub fn init() {
    unsafe {
        VM_TABLE.call_once(|| HashMap::new());
    }
}

pub fn table_delete_vm(vm_id: usize) {
    unsafe {
        VM_TABLE.get_mut().unwrap().remove(&vm_id).unwrap();
    }
}

pub fn table_insert_vm(vm_id: usize, vm: Arc<Mutex<VirtMach>>) {
    unsafe {
        assert!(VM_TABLE.get_mut().unwrap().insert(vm_id, vm).is_none());
    }
}

pub fn table_get_vm(vm_id: usize) -> Arc<Mutex<VirtMach>> {
    unsafe {
        VM_TABLE.get_mut().unwrap().get(&vm_id).unwrap().clone()
    }
}


static VM_ID_ALLOCATOR: AtomicUsize = AtomicUsize::new(0);

/// virtual machine state
pub enum VmState {
    Inactive,
    Active,
}

/// virtual machine
pub struct VirtMach {
    vm_id: usize,
    name: String,
    vcpus: Vec<Arc<VirtCpu>>,
    phy_mem: Vec<u8>, // 16M
    guest_phys_memory_set: GuestPhysMemorySet,
    entry: GuestPhysAddr,
}

impl VirtMach {
    fn set_vcpus(&mut self, vcpus: Vec<Arc<VirtCpu>>) {
        self.vcpus = vcpus;
    }

    pub fn nest_table_root(&self) -> HostPhysAddr {
        self.guest_phys_memory_set.nest_page_table_root()
    }

    pub fn new(vm_id: usize,
               name: String,
               phy_mem: Vec<u8>,
               guest_phys_memory_set: GuestPhysMemorySet,
               entry: GuestPhysAddr,
               cpu_affinities: Vec<CpuSet>,
    ) -> HyperResult<Arc<Mutex<Self>>> {
        let ntr = guest_phys_memory_set.nest_page_table_root();
        let vm = Arc::new(Mutex::new(VirtMach {
            vm_id,
            name,
            vcpus: vec![],
            phy_mem,
            guest_phys_memory_set,
            entry,
        }));

        let len = cpu_affinities.len();
        let mut vcpus = Vec::with_capacity(len);
        let mut iter = cpu_affinities.into_iter();
        for i in 0..len {
            if i == BSP_CPU_ID {
                vcpus.push(VirtCpu::new_bsp(
                    iter.next().ok_or(HyperError::Internal)?,
                    Arc::downgrade(&vm),
                    entry,
                    ntr,
                )?);
            } else {
                todo!()
                // vcpus.push(VirtCpu::new_ap())
            }
        }

        vm.lock().set_vcpus(vcpus);

        Ok(vm)
    }
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn vm_id(&self) -> usize {
        self.vm_id
    }
    pub fn start_bsp(&self) -> AxTaskRef {
        let bsp = self.vcpus[BSP_CPU_ID].clone();
        spawn_vcpu(bsp)
    }
    pub fn start_aps(&self, ap_start_entry: usize) {
        for (idx, ap) in self.vcpus.iter().enumerate() {
            if !idx == BSP_CPU_ID {
                spawn_vcpu(ap.clone());
            }
        }
    }
}


impl Display for VirtMach {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("VM {{ name: {}, id: {} }} ", self.name, self.vm_id))
    }
}


/// create vm and start its bsp vcpu
/// should close irq and disable preempt
pub fn boot_vm(conf: VmConfig) {
    let no_preempt_irq = SpinNoIrq::new(());
    let _guard = no_preempt_irq.lock();
    debug!("0");
    let VmConfig {
        name,
        cpu_affinities,
        bios_entry,
        bios_paddr,
        bios_size,
        guest_entry,
        guest_image_paddr,
        guest_image_size,
        guest_phys_memory_base,
        guest_phys_memory_size,
        mut guest_memory_region
    } = conf;

    // memory
    let mut phy_mem = vec![0; guest_phys_memory_size];
    load_guest_image(phy_mem.as_mut_slice(), bios_paddr, bios_entry, bios_size);
    load_guest_image(phy_mem.as_mut_slice(), guest_image_paddr, guest_entry, guest_image_size);

    guest_memory_region.push(GuestMemoryRegion {
        gpa: guest_phys_memory_base,
        hpa: virt_to_phys((phy_mem.as_ptr() as HostVirtAddr).into()).into(),
        size: guest_phys_memory_size,
        flags: MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE,
    });

    let mut gpm = GuestPhysMemorySet::new().unwrap();
    let root = gpm.nest_page_table_root();
    for r in guest_memory_region.into_iter() {
        gpm.map_region(r.into()).unwrap();
    }

    // vm
    let vm_id = VM_ID_ALLOCATOR.fetch_add(1, Ordering::Relaxed);
    let vm = VirtMach::new(vm_id, name, phy_mem, gpm, bios_entry, cpu_affinities).unwrap();
    table_insert_vm(vm_id, vm.clone());

    let guard = vm.lock();
    info!("boot {}", *guard);

    let tx = guard.start_bsp();
    tx.join();
}