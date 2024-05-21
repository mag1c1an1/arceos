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

pub fn setup_root_gpm() -> HyperResult<GuestPhysMemorySet> {
    let sys_config = HvSystemConfig::get();
    let cell_config = sys_config.root_cell.config();
    // trace!("cell_config:\n{:#x?}", cell_config);

    let mut gpm = GuestPhysMemorySet::new()?;
    debug!("create a new gpm");

    let hv_phys_start = sys_config.hypervisor_memory.phys_start as usize;
    let hv_phys_size = sys_config.hypervisor_memory.size as usize;
    let offset = hv_phys_start - hv_phys_start;

    gpm.map_region(
        GuestMemoryRegion {
            gpa: hv_phys_start as GuestPhysAddr,
            hpa: hv_phys_start as HostPhysAddr,
            size: hv_phys_size,
            flags: MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE,
        }
        .into(),
    )?;
    let mem_regions = cell_config.mem_regions();
    let mem_region_size = mem_regions.len();
    let mut index = 0;
    while (index < mem_region_size) {
        let region = mem_regions[index];
        let start_gpa = region.virt_start as usize;
        let start_hpa = region.phys_start as usize;
        let expected_flags = region.flags;
        let mut region_size = region.size as usize;
        let mut end_gpa = start_gpa + region_size;
        let offset = start_gpa - start_hpa;
        assert!(
            offset == 0,
            "Mem_regions from Root cell_config should have a zero offset!!!"
        );
        let mut next_i = index + 1;
        while next_i < mem_region_size {
            let cur_flags = mem_regions[next_i].flags;
            if mem_regions[next_i].virt_start as usize == end_gpa && cur_flags == expected_flags {
                let next_gpa = mem_regions[next_i].virt_start as usize;
                let next_size = mem_regions[next_i].size as usize;
                // debug!(
                //     "gpm mem region gpa:[{:#x}-{:#x}] is combined with [{:#x}-{:#x}]",
                //     next_gpa,
                //     next_gpa + next_size,
                //     start_gpa,
                //     start_gpa + region_size
                // );
                end_gpa += next_size;
                region_size += next_size;
                next_i += 1;
                index += 1;
            } else {
                break;
            }
        }
        gpm.map_region(
            GuestMemoryRegion {
                gpa: start_gpa as GuestPhysAddr,
                hpa: start_hpa as HostPhysAddr,
                size: region_size,
                flags: region.flags.into(),
            }
            .into(),
        )?;
        index += 1;
    }
    Ok(gpm)
}

pub fn init_root_gpm() -> HyperResult {
    let gpm = setup_root_gpm()?;
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
