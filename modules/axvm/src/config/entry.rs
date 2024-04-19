use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use spin::Mutex;

use axalloc::GlobalPage;
use axhal::mem::virt_to_phys;
use hypercraft::{GuestPhysAddr, HostPhysAddr, HostVirtAddr};
use page_table_entry::MappingFlags;

use crate::mm::{GuestMemoryRegion, GuestPhysMemorySet};
use crate::{Error, Result};

// VM_ID = 0 reserved for host Linux.
const CONFIG_VM_ID_START: usize = 1;
const CONFIG_VM_NUM_MAX: usize = 8;

struct VmConfigTable {
    entries: BTreeMap<usize, Arc<VMCfgEntry>>,
}

impl VmConfigTable {
    const fn new() -> VmConfigTable {
        VmConfigTable {
            entries: BTreeMap::new(),
        }
    }

    fn generate_vm_id(&mut self) -> Result<usize> {
        for i in CONFIG_VM_ID_START..CONFIG_VM_NUM_MAX {
            if !self.entries.contains_key(&i) {
                return Ok(i);
            }
        }
        Err(Error::OutOfRange)
    }

    fn remove_vm_by_id(&mut self, vm_id: usize) {
        if vm_id >= CONFIG_VM_NUM_MAX || !self.entries.contains_key(&vm_id) {
            error!("illegal vm id {}", vm_id);
        } else {
            self.entries.remove(&vm_id);
        }
    }
}

#[derive(Debug, Default)]
struct VMImgCfg {
    kernel_load_gpa: GuestPhysAddr,
    vm_entry_point: GuestPhysAddr,
    bios_load_gpa: GuestPhysAddr,
    ramdisk_load_gpa: GuestPhysAddr,

    kernel_load_hpa: HostPhysAddr,
    bios_load_hpa: HostPhysAddr,
    ramdisk_load_hpa: HostPhysAddr,
}

impl VMImgCfg {
    pub fn new(
        kernel_load_gpa: GuestPhysAddr,
        vm_entry_point: GuestPhysAddr,
        bios_load_gpa: GuestPhysAddr,
        ramdisk_load_gpa: GuestPhysAddr,
    ) -> VMImgCfg {
        VMImgCfg {
            kernel_load_gpa,
            vm_entry_point,
            bios_load_gpa,
            ramdisk_load_gpa,
            kernel_load_hpa: 0xdead_beef,
            bios_load_hpa: 0xdead_beef,
            ramdisk_load_hpa: 0xdead_beef,
        }
    }
}

#[derive(Debug)]
pub struct VMCfgEntry {
    vm_id: usize,
    name: String,
    cmdline: String,
    cpu_set: usize,

    img_cfg: VMImgCfg,

    ram_size: usize,
    ram_base_gpa: GuestPhysAddr,

    memory_regions: Vec<GuestMemoryRegion>,
    physical_pages: BTreeMap<usize, GlobalPage>,
    memory_set: Option<GuestPhysMemorySet>,
}

impl VMCfgEntry {
    pub fn new(
        name: String,
        cmdline: String,
        cpu_set: usize,
        kernel_load_gpa: GuestPhysAddr,
        vm_entry_point: GuestPhysAddr,
        bios_load_gpa: GuestPhysAddr,
        ramdisk_load_gpa: GuestPhysAddr,
        ram_size: usize,
        ram_base_gpa: GuestPhysAddr,
    ) -> Self {
        VMCfgEntry {
            vm_id: 0xdeaf_beef,
            name,
            cmdline,
            cpu_set,
            ram_size,
            img_cfg: VMImgCfg::new(
                kernel_load_gpa,
                vm_entry_point,
                bios_load_gpa,
                ramdisk_load_gpa,
            ),
            ram_base_gpa,
            memory_regions: Vec::new(),
            physical_pages: BTreeMap::new(),
            memory_set: None,
        }
    }

    pub fn get_cpu_set(&self) -> usize {
        self.cpu_set
    }

    pub fn get_vm_entry(&self) -> GuestPhysAddr {
        self.img_cfg.vm_entry_point
    }

    pub fn add_physical_pages(&mut self, index: usize, pages: GlobalPage) {
        self.physical_pages.insert(index, pages);
    }

    fn memory_region_editor<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Vec<GuestMemoryRegion>),
    {
        f(&mut self.memory_regions)
    }

    pub fn set_up_memory_region(&mut self) {
        // Currenly we just use physical_pages[0]
        // because we only need to allocate one physical mem region for Guest VM RAM.
        let ram_base_hpa = self
            .physical_pages
            .get(&0)
            .expect("physical memory space for Guest VM not allocated")
            .start_paddr(virt_to_phys)
            .as_usize();

        self.memory_regions.push(GuestMemoryRegion {
            gpa: self.ram_base_gpa,
            hpa: ram_base_hpa,
            size: self.ram_size,
            flags: MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE,
        });

        self.memory_region_editor(super::nimbos_cfg_def::nimbos_memory_regions_setup);
    }

    pub fn generate_guest_phys_memory_set(&self) -> Result<GuestPhysMemorySet> {
        info!("Create VM [{}] nested page table", self.vm_id);

        // create nested page table and add mapping
        let mut gpm = GuestPhysMemorySet::new()?;
        for r in &self.memory_regions {
            gpm.map_region(r.clone().into())?;
        }
        Ok(gpm)
    }

    fn addr_in_memory_range(&self, addr: GuestPhysAddr) -> bool {
        let physical_mem_range = self.ram_base_gpa..(self.ram_base_gpa + self.ram_size);
        return physical_mem_range.contains(&addr);
    }

    pub fn get_img_load_info(&mut self) -> (HostPhysAddr, HostPhysAddr, HostPhysAddr) {
        // Currenly we just use physical_pages[0]
        // because we only need to allocate one physical mem region for Guest VM RAM.
        let ram_base_hpa = self
            .physical_pages
            .get(&0)
            .expect("physical memory space for Guest VM not allocated")
            .start_paddr(virt_to_phys)
            .as_usize();

        if self.addr_in_memory_range(self.img_cfg.bios_load_gpa) {
            self.img_cfg.bios_load_hpa =
                self.img_cfg.bios_load_gpa - self.ram_base_gpa + ram_base_hpa;
        } else {
            warn!(
                "Guest VM bios load gpa {:#x} not in memory range {:#x?}",
                self.img_cfg.bios_load_gpa,
                self.ram_base_gpa..(self.ram_base_gpa + self.ram_size),
            );
            self.img_cfg.bios_load_hpa = 0;
        }

        if self.addr_in_memory_range(self.img_cfg.kernel_load_gpa) {
            self.img_cfg.kernel_load_hpa =
                self.img_cfg.kernel_load_gpa - self.ram_base_gpa + ram_base_hpa;
        } else {
            warn!(
                "Guest VM kernel load gpa {:#x} not in memory range {:#x?}",
                self.img_cfg.kernel_load_gpa,
                self.ram_base_gpa..(self.ram_base_gpa + self.ram_size),
            );
            self.img_cfg.kernel_load_hpa = 0;
        }

        if self.addr_in_memory_range(self.img_cfg.ramdisk_load_gpa) {
            self.img_cfg.ramdisk_load_hpa =
                self.img_cfg.ramdisk_load_gpa - self.ram_base_gpa + ram_base_hpa;
        } else {
            self.img_cfg.ramdisk_load_hpa = 0;
        }

        debug!(
            "memory range {:#?} ram_base_hpa {:#x}",
            self.ram_base_gpa..(self.ram_base_gpa + self.ram_size),
            ram_base_hpa
        );
        debug!(
            "bios_load_hpa {:#x} kernel_load_hpa {:#x} ramdisk_load_hpa {:#x}",
            self.img_cfg.bios_load_hpa,
            self.img_cfg.kernel_load_hpa,
            self.img_cfg.ramdisk_load_hpa,
        );

        (
            self.img_cfg.bios_load_hpa,
            self.img_cfg.kernel_load_hpa,
            self.img_cfg.ramdisk_load_hpa,
        )
    }
}

static GLOBAL_VM_CFG_TABLE: Mutex<VmConfigTable> = Mutex::new(VmConfigTable::new());

pub fn vm_cfg_entry(vm_id: usize) -> Option<Arc<VMCfgEntry>> {
    let vm_configs = GLOBAL_VM_CFG_TABLE.lock();
    return vm_configs.entries.get(&vm_id).cloned();
}

/* Add VM config entry to DEF_VM_CONFIG_TABLE
 *
 * @param[in] vm_cfg_entry: new added VM config entry.
 * @param[out] vm_id: the VM id of newly added VM.
 */
pub fn vm_cfg_add_vm_entry(mut vm_cfg_entry: VMCfgEntry) -> Result<usize> {
    let mut vm_configs = GLOBAL_VM_CFG_TABLE.lock();
    let vm_id = vm_configs.generate_vm_id()?;
    vm_cfg_entry.vm_id = vm_id;
    vm_configs.entries.insert(vm_id, Arc::new(vm_cfg_entry));
    Ok(vm_id)
}
