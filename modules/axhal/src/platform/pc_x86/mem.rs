// TODO: get memory regions from multiboot info.

use crate::mem::*;

/// Number of physical memory regions.
pub(crate) fn memory_regions_num() -> usize {
    cfg_if::cfg_if! {
        if #[cfg(feature="hv")] {
    common_memory_regions_num() + 3
        } else {
    common_memory_regions_num() + 2
        }
    }
}

/// Returns the physical memory region at the given index, or [`None`] if the
/// index is out of bounds.
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
