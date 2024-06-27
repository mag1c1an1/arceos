use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use hypercraft::{GuestPhysAddr, HostPhysAddr};
use page_table_entry::MappingFlags;
use crate::utils::CpuSet;
use crate::hv::mm::GuestMemoryRegion;


pub const MAX_VCPUS: usize = 4;
pub const BSP_CPU_ID: usize = 0;


pub struct VmConfig {
    pub name: String,
    pub cpu_affinities: Vec<CpuSet>,
    pub bios_entry: GuestPhysAddr,
    pub bios_paddr: HostPhysAddr,
    pub bios_size: usize,
    pub guest_entry: GuestPhysAddr,
    pub guest_image_paddr: HostPhysAddr,
    pub guest_image_size: usize,
    pub guest_phys_memory_base: GuestPhysAddr,
    pub guest_phys_memory_size: usize,
    pub guest_memory_region: Vec<GuestMemoryRegion>,
}

pub fn arceos_config() -> VmConfig {
    let mut cpu_affinities = Vec::new();
    for i in 0..4 {
        let mut affinity = CpuSet::new_empty();
        affinity.add(i % 2);
        cpu_affinities.push(affinity);
    }

    let guest_memory_region = vec![
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

    VmConfig {
        name: String::from("ArceOS"),
        cpu_affinities,
        bios_entry: 0x8000,
        bios_paddr: 0x7400_0000,
        bios_size: 0x1000,
        guest_entry: 0x20_0000,
        guest_image_paddr: 0x7400_1000,
        guest_image_size: 0x10_0000, // 1M
        guest_phys_memory_base: 0,
        guest_phys_memory_size: 0x800_0000, // 16M
        guest_memory_region,
    }
}
