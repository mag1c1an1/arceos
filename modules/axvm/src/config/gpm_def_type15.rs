use axhal::config::{CellConfig, HvSystemConfig};
use hypercraft::{GuestPageTableTrait, GuestPhysAddr, HostPhysAddr, HostVirtAddr, HyperCraftHal};
use memory_addr::align_down_4k;
use page_table_entry::MappingFlags;

use crate::mm::{GuestMemoryRegion, GuestPhysMemorySet};
use crate::{Error, GuestPageTable, Result as HyperResult};

static ROOT_GPM: spin::Once<GuestPhysMemorySet> = spin::Once::new();

pub fn root_gpm() -> &'static GuestPhysMemorySet {
    ROOT_GPM.get().expect("Uninitialized root gpm!")
}

pub fn setup_gpm() -> HyperResult<GuestPhysMemorySet> {
    let sys_config = HvSystemConfig::get();
    let cell_config = sys_config.root_cell.config();
    // trace!("cell_config:\n{:#x?}", cell_config);

    let mut gpm = GuestPhysMemorySet::new()?;
    debug!("create a new gpm");

    let hv_phys_start = sys_config.hypervisor_memory.phys_start as usize;
    let hv_phys_size = sys_config.hypervisor_memory.size as usize;
    let offset = hv_phys_start - hv_phys_start;
    trace!(
        "gpm mapped gpa:{:#x} hpa: {:#x} offset:{:#x} size: {:#x}",
        hv_phys_start, hv_phys_start, offset, hv_phys_size
    );
    
    gpm.map_region(GuestMemoryRegion {
        gpa: hv_phys_start as GuestPhysAddr,
        hpa: hv_phys_start as HostPhysAddr,
        size: hv_phys_size,
        flags: MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE,
    }.into())?;
    for region in cell_config.mem_regions() {
        let start_gpa = region.virt_start as usize;
        let start_hpa = region.phys_start as usize;
        let region_size = region.size as usize;
        let offset = start_gpa - start_hpa;
        trace!("gpm mapped gpa:{:#x} hpa: {:#x} offset:{:#x} size:{:#x}", start_gpa, start_hpa, offset, region_size);
        gpm.map_region(GuestMemoryRegion {
            gpa: start_gpa as GuestPhysAddr,
            hpa: start_hpa as HostPhysAddr,
            size: region_size,
            flags: region.flags.into(),
        }.into())?;
    }
    Ok(gpm)
}

pub fn init_gpm() -> HyperResult {
    let gpm = setup_gpm()?;
    ROOT_GPM.call_once(|| gpm);
    Ok(())
}