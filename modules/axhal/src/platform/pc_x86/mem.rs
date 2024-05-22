// TODO: get memory regions from multiboot info.

use crate::mem::*;

#[cfg(feature = "type1_5")]
use crate::platform::config::HvSystemConfig;
#[cfg(feature = "type1_5")]
use crate::platform::header::HvHeader;
#[cfg(feature = "type1_5")]
use lazy_init::LazyInit;
#[cfg(feature = "type1_5")]
static mmio_num: LazyInit<usize> = LazyInit::new();

/// Number of physical memory regions.
pub(crate) fn memory_regions_num() -> usize {
    cfg_if::cfg_if! {
        if #[cfg(feature="type1_5")] {
            *mmio_num
        } else if #[cfg(feature="hv")] {
            common_memory_regions_num() + 3
        } else {
            common_memory_regions_num() + 2
        }
    }
}

/// Returns the physical memory region at the given index, or [`None`] if the
/// index is out of bounds.
#[cfg(not(feature = "type1_5"))]
pub(crate) fn memory_region_at(idx: usize) -> Option<MemRegion> {
    let num = common_memory_regions_num();
    if idx == 0 {
        // low physical memory
        Some(MemRegion {
            paddr: PhysAddr::from(0),
            size: 0x9f000,
            flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::WRITE,
            name: "low memory",
        })
    } else if idx <= num {
        common_memory_region_at(idx - 1)
    } else if idx == num + 1 {
        // free memory
        extern "C" {
            fn ekernel();
        }
        let start = virt_to_phys((ekernel as usize).into()).align_up_4k();
        let end = PhysAddr::from(axconfig::PHYS_MEMORY_END).align_down_4k();
        Some(MemRegion {
            paddr: start,
            size: end.as_usize() - start.as_usize(),
            flags: MemRegionFlags::FREE | MemRegionFlags::READ | MemRegionFlags::WRITE,
            name: "free memory",
        })
    } else if idx == num + 2 {
        // Mapped for shared memory.
        Some(MemRegion {
            paddr: PhysAddr::from(axconfig::PHYS_MEMORY_END),
            size: axconfig::SYSCALL_DATA_BUF_SIZE + axconfig::SYSCALL_QUEUE_BUF_SIZE,
            flags: MemRegionFlags::RESERVED
                | MemRegionFlags::DEVICE
                | MemRegionFlags::READ
                | MemRegionFlags::WRITE,
            name: "resered shared memory",
        })
    } else {
        None
    }
}

/// Returns the physical memory region at the given index, or [`None`] if the
/// index is out of bounds.
#[cfg(feature = "type1_5")]
pub(crate) fn memory_region_at(idx: usize) -> Option<MemRegion> {
    let num = memory_regions_num();
    let header = HvHeader::get();
    let sys_config = HvSystemConfig::get();
    let cell_config = sys_config.root_cell.config();
    let hv_phys_start = sys_config.hypervisor_memory.phys_start as usize;
    let hv_phys_size = sys_config.hypervisor_memory.size as usize;

    if idx == 0 {
        Some(MemRegion {
            paddr: PhysAddr::from(hv_phys_start),
            size: header.core_size,
            flags: MemRegionFlags::READ | MemRegionFlags::WRITE | MemRegionFlags::EXECUTE,
            name: "hv kernel memory",
        })
    } else if idx == 1 {
        Some(MemRegion {
            paddr: PhysAddr::from(hv_phys_start + header.core_size),
            size: hv_phys_size - header.core_size,
            flags: MemRegionFlags::READ | MemRegionFlags::WRITE,
            name: "hv free memory",
        })
    } else if idx < num {
        let mem_regions = cell_config.mem_regions();
        let region = mem_regions[idx - 2];
        let mut region_paddr = region.phys_start as usize;
        let mut region_size = region.size as usize;
        let region_flag = region.flags.into();
        if (region_paddr..region_paddr + region_size).contains(&hv_phys_start) {
            if region_paddr == hv_phys_start {
                region_paddr = hv_phys_start + hv_phys_size;
                region_size -= hv_phys_size;
            } else {
                region_size = hv_phys_start - region_paddr;
            }
            debug!(
                "Linux region {:#x?} contains hv memory, decrease to [{:#x}-{:#x}]",
                region,
                region_paddr,
                region_paddr + region_size,
            );
        }
        Some(MemRegion {
            paddr: PhysAddr::from(region_paddr),
            size: region_size,
            flags: region_flag,
            name: "Host Linux region",
        })
    } else {
        None
    }
}
/// init mmio_num
#[cfg(feature = "type1_5")]
pub(crate) fn init_mmio_num() {
    let sys_config = HvSystemConfig::get();
    let cell_config = sys_config.root_cell.config();
    let num = cell_config.mem_regions().len() + 2;
    // info!("mmio_num = {}", num);
    mmio_num.init_by(num);
}
