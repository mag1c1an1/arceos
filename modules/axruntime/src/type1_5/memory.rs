use super::consts::{
    HV_BASE, HV_HEAP_SIZE, MACHINE_ALIGN,
    free_memory_start, hv_end
};
use super::config::{HvSystemConfig, MemFlags};
use super::header::HvHeader;

use axhal::mem::{memory_regions, phys_to_virt};
use axhal::paging::PageTable;
use hypercraft::HyperResult;
use memory_addr::{PhysAddr, VirtAddr};
use page_table_entry::MappingFlags;

use spin::{Once, RwLock};

/// Page table used for hypervisor.
static HV_PT: Once<RwLock<PageTable>> = Once::new();

pub fn hv_page_table<'a>() -> &'a RwLock<PageTable> {
    HV_PT.get().expect("Uninitialized hypervisor page table!")
}

pub fn init_type15_allocator() {
    let mem_pool_start = free_memory_start();
    let mem_pool_end = hv_end().align_down_4k();

    let mem_pool_size = mem_pool_end.as_usize() - mem_pool_start.as_usize();
    info!("global_init start:{:x}, end:{:x}.",mem_pool_start,mem_pool_end);
    axalloc::global_init(mem_pool_start.as_usize(), mem_pool_size);
    
    let header = HvHeader::get();
    let sys_config = HvSystemConfig::get();
    let cell_config = sys_config.root_cell.config();
    let hv_phys_start = sys_config.hypervisor_memory.phys_start as usize;
    let hv_phys_size = sys_config.hypervisor_memory.size as usize;
    info!("create PageTable.");
    let mut page_table = PageTable::try_new().unwrap();
    
    page_table.map_region(
        VirtAddr::from(HV_BASE),
        PhysAddr::from(hv_phys_start),
        header.core_size,
        MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE,
        false,
    );
    info!("map_region {:x},{:x},{:x},{:?},{}.",
    HV_BASE,
    hv_phys_start,
    header.core_size,
    MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE,
    false);
    page_table.map_region(
        (HV_BASE + header.core_size).into(),
        (hv_phys_start + header.core_size).into(),
        hv_phys_size - header.core_size,
        MappingFlags::READ | MappingFlags::WRITE,
        false,
    );
    info!("map_region {:x},{:x},{:x},{:?},{}.",
        HV_BASE + header.core_size,
        hv_phys_start + header.core_size,
        hv_phys_size - header.core_size,
        MappingFlags::READ | MappingFlags::WRITE,
        false);
    // Map all guest RAM to directly access in hypervisor.
    for region in cell_config.mem_regions() {
        let flags = region.flags; 
        if flags.contains(MemFlags::DMA) {
            let hv_virt_start = phys_to_virt(PhysAddr::from(region.virt_start as _));
            if hv_virt_start < VirtAddr::from(region.virt_start as _) {
                let virt_start = region.virt_start;
                panic!(
                        "Guest physical address {:#x} is too large",
                        virt_start
                );
            }
            page_table.map_region(
                hv_virt_start,
                PhysAddr::from(region.phys_start as _),
                region.size as usize,
                MappingFlags::READ | MappingFlags::WRITE,
                false
            );
            info!("map_region {:x},{:x},{:x},{:?},{}.",
            hv_virt_start.as_usize(),
            region.phys_start as usize,
            region.size as usize,
            MappingFlags::READ | MappingFlags::WRITE,
            false);
        }
    }
    info!("Hypervisor page table init end.");
    // info!("Hypervisor virtual memory set: {:#x?}", page_table);
    
    HV_PT.call_once(|| RwLock::new(page_table));
}

pub fn activate_hv_pt() {
    let page_table = HV_PT.get().expect("Uninitialized hypervisor page table!");
    unsafe { axhal::arch::write_page_table_root(page_table.read().root_paddr()) };
}

pub fn init_hv_page_table() -> Result<(), axhal::paging::PagingError> {
    let header = HvHeader::get();
    let sys_config = HvSystemConfig::get();
    info!("Hypervisor header: {:#x?}", header);
    // info!("Hvconfig: {:#x?}", sys_config);
    let cell_config = sys_config.root_cell.config();
    let hv_phys_start = sys_config.hypervisor_memory.phys_start as usize;
    let hv_phys_size = sys_config.hypervisor_memory.size as usize;

    let mut hv_pt = PageTable::try_new()?;

    // Map hypervisor memory.
    // TODO: Fine-grained permissions setting
    let mem_flag = MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE;
    hv_pt.map_region(
        VirtAddr::from(HV_BASE),
        PhysAddr::from(hv_phys_start),
        header.core_size,
        mem_flag.into(),
        true,   // allow huge?
    )?;
    debug!("hv pt mapping: {:#x?} -> {:#x?} size: {:#x}", HV_BASE, hv_phys_start, header.core_size);
    let mem_flag = MemFlags::READ | MemFlags::WRITE;
    hv_pt.map_region(
        VirtAddr::from(HV_BASE + header.core_size),
        PhysAddr::from(hv_phys_start + header.core_size),
        hv_phys_size - header.core_size,
        mem_flag.into(),
        true,
    )?;
    debug!("hv pt mapping: {:#x?} -> {:#x?} size: {:#x}", HV_BASE + header.core_size, hv_phys_start + header.core_size, hv_phys_size - header.core_size);
    /*
    hv_pt.insert(MemoryRegion::new_with_offset_mapper(
        HV_BASE,
        hv_phys_start,
        header.core_size,
        MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE,
    ))?;
    // Map per-CPU data, configurations & free page pool.
    hv_pt.insert(MemoryRegion::new_with_offset_mapper(
        HV_BASE + header.core_size,
        hv_phys_start + header.core_size,
        hv_phys_size - header.core_size,
        MemFlags::READ | MemFlags::WRITE,
    ))?;
    */
    // Map all guest RAM to directly access in hypervisor.
    for region in cell_config.mem_regions() {
        let flags = region.flags;
        if flags.contains(MemFlags::DMA) {
            let phys_start = region.phys_start;
            let hv_virt_start = phys_to_virt(PhysAddr::from(phys_start as usize));
            let size = region.size;
            debug!("hv pt mapping: {:#x?} -> {:#x?} size: {:#x}", hv_virt_start, phys_start, size);
            if hv_virt_start < VirtAddr::from(phys_start as usize) {
                return Err(
                    axhal::paging::PagingError::NoMemory
                );
            }
            hv_pt.map_region(
                hv_virt_start,
                PhysAddr::from(phys_start as usize),
                size as usize,
                mem_flag.into(),
                true,
            )?;
            /* 
            hv_pt.insert(MemoryRegion::new_with_offset_mapper(
                hv_virt_start,
                region.phys_start as HostPhysAddr,
                region.size as usize,
                MemFlags::READ | MemFlags::WRITE,
            ))?;
            */
        }
    }
    info!("Hypervisor page table init end.");

    HV_PT.call_once(|| RwLock::new(hv_pt));
    let pt = HV_PT.get().unwrap().read();
    unsafe { axhal::arch::write_page_table_root(pt.root_paddr()) };

    // debug!("Hypervisor virtual memory set: {:#x?}", pt);
    Ok(())
}
