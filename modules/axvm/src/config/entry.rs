use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;

use spin::Mutex;

use crate::{Error, Result};
use axalloc::GlobalPage;

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

#[derive(Debug)]
pub struct VMCfgEntry {
    vm_id: usize,

    name: String,

    cmdline: String,

    vm_entry: usize,

    cpu_set: usize,

    // Todo: combine these into `VmImageConfig`.
    bios_loaded_pages: Option<GlobalPage>,

    kernel_img_loaded_pages: Option<GlobalPage>,

    ramdisk_img_loaded_pages: Option<GlobalPage>,
}

impl VMCfgEntry {
    pub fn new(name: String, cmdline: String, vm_entry: usize, cpu_set: usize) -> Self {
        VMCfgEntry {
            vm_id: 0xdeaf_beef,
            name,
            cmdline,
            vm_entry,
            cpu_set,
            bios_loaded_pages: None,
            kernel_img_loaded_pages: None,
            ramdisk_img_loaded_pages: None,
        }
    }

    pub fn get_cpu_set(&self) -> usize {
        self.cpu_set
    }

    pub fn set_bios_loaded_pages(&mut self, pages: GlobalPage) {
        debug_assert!(
            self.bios_loaded_pages.is_none(),
            "bios_loaded_pages already set"
        );
        self.bios_loaded_pages = Some(pages);
    }

    pub fn get_bios_loaded_pages(&self) -> Option<&GlobalPage> {
        self.bios_loaded_pages.as_ref()
    }

    pub fn set_kernel_img_loaded_pages(&mut self, pages: GlobalPage) {
        debug_assert!(
            self.kernel_img_loaded_pages.is_none(),
            "kernel_img_loaded_pages already set"
        );
        self.kernel_img_loaded_pages = Some(pages);
    }

    pub fn get_kernel_img_loaded_pages(&self) -> Option<&GlobalPage> {
        self.kernel_img_loaded_pages.as_ref()
    }

    pub fn set_ramdisk_img_loaded_pages(&mut self, pages: GlobalPage) {
        debug_assert!(
            self.ramdisk_img_loaded_pages.is_none(),
            "ramdisk_img_loaded_pages already set"
        );
        self.ramdisk_img_loaded_pages = Some(pages);
    }

    pub fn get_ramdisk_img_loaded_pages(&self) -> Option<&GlobalPage> {
        self.ramdisk_img_loaded_pages.as_ref()
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
