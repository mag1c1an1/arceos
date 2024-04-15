use axhal::config::{CellConfig, HvSystemConfig};
use hypercraft::{GuestPageTableTrait, GuestPhysAddr, HostPhysAddr, HostVirtAddr, HyperCraftHal};
use memory_addr::align_down_4k;
use page_table_entry::MappingFlags;

use crate::mm::{GuestMemoryRegion, GuestPhysMemorySet};
use crate::{phys_to_virt, virt_to_phys};
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
        hv_phys_start,
        hv_phys_start,
        offset,
        hv_phys_size
    );

    gpm.map_region(
        GuestMemoryRegion {
            gpa: hv_phys_start as GuestPhysAddr,
            hpa: hv_phys_start as HostPhysAddr,
            size: hv_phys_size,
            flags: MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE,
        }
        .into(),
    )?;
    for region in cell_config.mem_regions() {
        let start_gpa = region.virt_start as usize;
        let start_hpa = region.phys_start as usize;
        let region_size = region.size as usize;
        let offset = start_gpa - start_hpa;
        trace!(
            "gpm mapped gpa:{:#x} hpa: {:#x} offset:{:#x} size:{:#x}",
            start_gpa,
            start_hpa,
            offset,
            region_size
        );
        gpm.map_region(
            GuestMemoryRegion {
                gpa: start_gpa as GuestPhysAddr,
                hpa: start_hpa as HostPhysAddr,
                size: region_size,
                flags: region.flags.into(),
            }
            .into(),
        )?;
    }
    Ok(gpm)
}

pub fn init_gpm() -> HyperResult {
    let gpm = setup_gpm()?;
    ROOT_GPM.call_once(|| gpm);
    Ok(())
}

const GUEST_PHYS_MEMORY_SIZE: usize = 0x100_0000; // 16M
const GUEST_PHYS_MEMORY_BASE: GuestPhysAddr = 0;

const NIMBOS_BIOS_ENTRY: GuestPhysAddr = 0x8000;
const NIMBOS_GUEST_ENTRY: GuestPhysAddr = 0x20_0000;

#[repr(align(4096))]
pub(super) struct AlignedMemory<const LEN: usize>([u8; LEN]);

pub(super) static mut GUEST_PHYS_MEMORY: AlignedMemory<GUEST_PHYS_MEMORY_SIZE> =
    AlignedMemory([0; GUEST_PHYS_MEMORY_SIZE]);

fn gpa_as_mut_ptr(guest_paddr: GuestPhysAddr) -> *mut u8 {
    let offset = unsafe { &(GUEST_PHYS_MEMORY) as *const _ as usize };
    let host_vaddr = guest_paddr + offset;
    host_vaddr as *mut u8
}

#[cfg(target_arch = "x86_64")]
fn load_guest_image(hpa: HostPhysAddr, load_gpa: GuestPhysAddr, size: usize) {
    let image_ptr = usize::from(phys_to_virt(hpa.into())) as *const u8;
    // let image = unsafe { core::slice::from_raw_parts(image_ptr, 110) };
    // info!("first 110 byte: {:#x?}", image);
    let image = unsafe { core::slice::from_raw_parts(image_ptr, size) };

    trace!(
        "loading to guest memory: host {:#x} to guest {:#x}, size {:#x}",
        image_ptr as usize,
        load_gpa,
        size
    );

    unsafe {
        core::slice::from_raw_parts_mut(gpa_as_mut_ptr(load_gpa), size).copy_from_slice(image)
    }
}

#[cfg(target_arch = "x86_64")]
pub fn setup_nimbos_gpm(
    bios_paddr: usize,
    bios_size: usize,
    guest_image_paddr: usize,
    guest_image_size: usize,
) -> HyperResult<GuestPhysMemorySet> {
    // copy BIOS and guest images
    load_guest_image(bios_paddr, NIMBOS_BIOS_ENTRY, bios_size);
    load_guest_image(guest_image_paddr, NIMBOS_GUEST_ENTRY, guest_image_size);

    info!("1");
    // create nested page table and add mapping
    let mut gpm = GuestPhysMemorySet::new()?;
    let guest_memory_regions = [
        GuestMemoryRegion {
            // Low RAM
            gpa: GUEST_PHYS_MEMORY_BASE,
            hpa: virt_to_phys((gpa_as_mut_ptr(GUEST_PHYS_MEMORY_BASE) as HostVirtAddr).into())
                .into(),
            size: GUEST_PHYS_MEMORY_SIZE,
            flags: MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE,
        },
        GuestMemoryRegion {
            // syscall forwarder region
            gpa: 0x6700_0000,
            hpa: 0x700_0000,
            size: 0x0100_0000,
            flags: MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE,
        },
        GuestMemoryRegion {
            // PCI
            gpa: 0x8000_0000,
            hpa: 0x8000_0000,
            size: 0x1000_0000,
            flags: MappingFlags::READ | MappingFlags::WRITE,
        },
        // GuestMemoryRegion {
        //     gpa: 0xfe00_0000,
        //     hpa: 0xfe00_0000,
        //     size: 0x1_0000,
        //     flags: MappingFlags::READ | MappingFlags::WRITE | MappingFlags::DEVICE,
        // },
        // GuestMemoryRegion {
        //     gpa: 0xfeb0_0000,
        //     hpa: 0xfeb0_0000,
        //     size: 0x10_0000,
        //     flags: MappingFlags::READ | MappingFlags::WRITE | MappingFlags::DEVICE,
        // },
        GuestMemoryRegion {
            // IO APIC
            gpa: 0xfec0_0000,
            hpa: 0xfec0_0000,
            size: 0x1000,
            flags: MappingFlags::READ | MappingFlags::WRITE,
        },
        GuestMemoryRegion {
            // HPET
            gpa: 0xfed0_0000,
            hpa: 0xfed0_0000,
            size: 0x1000,
            flags: MappingFlags::READ | MappingFlags::WRITE,
        },
        GuestMemoryRegion {
            // Local APIC
            gpa: 0xfee0_0000,
            hpa: 0xfee0_0000,
            size: 0x1000,
            flags: MappingFlags::READ | MappingFlags::WRITE,
        },
        // SCF: memory region for shared memory should be configged here.
    ];
    for r in guest_memory_regions.into_iter() {
        gpm.map_region(r.into())?;
    }
    Ok(gpm)
}
