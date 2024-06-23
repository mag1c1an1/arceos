//! Abstraction of a virtual machine

use alloc::vec;
use alloc::vec::Vec;
use core::ptr::addr_of;
use core::sync::atomic::{AtomicUsize, Ordering};
use hashbrown::HashMap;
use lazy_static::lazy_static;
use spin::{Mutex};
use axhal::mem::{MemRegion, virt_to_phys, VirtAddr};
use crate::hv::mm::{GuestMemoryRegion, GuestPhysMemorySet, load_guest_image};

mod config;

pub use config::VmConfig;
pub use config::nimbos_config;
use hypercraft::{HostVirtAddr, HyperError, HyperResult, PerCpu, VmxExitInfo};
use page_table_entry::MappingFlags;
use crate::hv::{HyperCraftHalImpl, vmx};

/// global virtual machine hashmap
lazy_static! {
pub static ref VM_TABLE: Mutex<HashMap<usize, VirtMach>> = Mutex::new(HashMap::new());
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
    // vcpus:
    phy_mem: Vec<u8>, // 16M
    guest_phys_memory_set: GuestPhysMemorySet,
}

impl VirtMach {
    fn new(vm_id: usize, phy_mem: Vec<u8>, guest_phys_memory_set: GuestPhysMemorySet) -> Self {
        Self {
            vm_id,
            phy_mem,
            guest_phys_memory_set,
        }
    }
}


pub fn create_vm(conf: VmConfig) -> HyperResult<usize> {
    debug!("0");
    let VmConfig {
        name,
        cpu_affinity,
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

    let mut phy_mem = vec![0; guest_phys_memory_size];
    debug!("mem {:p}",phy_mem.as_ptr());
    load_guest_image(phy_mem.as_mut_slice(), bios_paddr, bios_entry, bios_size);
    load_guest_image(phy_mem.as_mut_slice(), guest_image_paddr, guest_entry, guest_image_size);

    guest_memory_region.push(GuestMemoryRegion {
        gpa: guest_phys_memory_base,
        hpa: virt_to_phys((phy_mem.as_ptr() as HostVirtAddr).into()).into(),
        size: guest_phys_memory_size,
        flags: MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE,
    });

    let mut gpm = GuestPhysMemorySet::new()?;
    let root = gpm.nest_page_table_root();

    for r in guest_memory_region.into_iter() {
        gpm.map_region(r.into())?;
    }


    debug!("0");
    let vm_id = VM_ID_ALLOCATOR.fetch_add(1, Ordering::Relaxed);
    debug!("1");
    let vm = VirtMach::new(vm_id, phy_mem, gpm);
    debug!("2");
    let ptr = vm.phy_mem.as_ptr();
    debug!("ptr {:p}", ptr);
    let mut guard = VM_TABLE.lock();
    guard.insert(vm_id, vm);
    let vm = guard.get_mut(&vm_id).unwrap();
    let new_ptr = vm.phy_mem.as_ptr();
    error!("new ptr {:p}",new_ptr);
    let mut p = PerCpu::<HyperCraftHalImpl>::new(0);
    p.hardware_enable()?;
    let mut vcpu = p.create_vcpu(bios_entry, root)?;

    vcpu.bind_to_current_cpu().unwrap();

    loop {
        match vcpu.run() {
            None => {}
            Some(_) => {
                match vmx::vmexit_handler(&mut vcpu) {
                    Ok(_) => { continue }
                    Err(e) => {
                        if matches!(e, HyperError::Shutdown) {
                            debug!("shutdown");
                            break;
                        } else {
                            panic!("hyper error{:?}", e);
                        }
                    }
                }
            }
        }
    }

    Ok(vm_id)
}

