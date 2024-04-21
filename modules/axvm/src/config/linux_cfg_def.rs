use alloc::vec::Vec;

use page_table_entry::MappingFlags;

use crate::mm::GuestMemoryRegion;
use crate::config::GUEST_PHYS_MEMORY_SIZE;
use crate::config::GUEST_PHYS_MEMORY_BASE;

// See `apps/hv/guest/vlbl/virt_int.c`
pub fn linux_memory_regions_setup(regions: &mut Vec<GuestMemoryRegion>) {
    let guest_memory_regions = [
        // 0x0000_0000 ~ 0x0100_0000
        GuestMemoryRegion {
            // Low RAM
            gpa: GUEST_PHYS_MEMORY_BASE,
            hpa: 0,
            size: GUEST_PHYS_MEMORY_SIZE,
            flags: MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE,
        },
        // 0x0100_0000 ~ 0x1000_0000 (16m ~ 256m)
        GuestMemoryRegion {
            // Low RAM2
            gpa: 0x100_0000,
            hpa: 0,
            size: 0xf00_0000,
            flags: MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE,
        },
        // 0x7000_0000 ~ 0x8000_0000
        GuestMemoryRegion {
            // RAM
            gpa: 0x7000_0000,
            hpa: 0,
            size: 0x1000_0000,
            flags: MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE,
        },
        GuestMemoryRegion {
            // PCI
            gpa: 0x8000_0000,
            hpa: 0x8000_0000,
            size: 0x1000_0000,
            flags: MappingFlags::READ | MappingFlags::WRITE | MappingFlags::DEVICE,
        },
        GuestMemoryRegion {
            gpa: 0xfe00_0000,
            hpa: 0xfe00_0000,
            size: 0x1_0000,
            flags: MappingFlags::READ | MappingFlags::WRITE | MappingFlags::DEVICE,
        },
        GuestMemoryRegion {
            gpa: 0xfeb0_0000,
            hpa: 0xfeb0_0000,
            size: 0x10_0000,
            flags: MappingFlags::READ | MappingFlags::WRITE | MappingFlags::DEVICE,
        },
        GuestMemoryRegion {
            // IO APIC
            gpa: 0xfec0_0000,
            hpa: 0xfec0_0000,
            size: 0x1000,
            flags: MappingFlags::READ | MappingFlags::WRITE | MappingFlags::DEVICE,
        },
        GuestMemoryRegion {
            // HPET
            gpa: 0xfed0_0000,
            hpa: 0xfed0_0000,
            size: 0x1000,
            flags: MappingFlags::READ | MappingFlags::WRITE | MappingFlags::DEVICE,
        },
        GuestMemoryRegion {
            // Local APIC
            gpa: 0xfee0_0000,
            hpa: 0xfee0_0000,
            size: 0x1000,
            flags: MappingFlags::READ | MappingFlags::WRITE | MappingFlags::DEVICE,
        },
    ];
    for r in guest_memory_regions {
        regions.push(r);
    }
}
