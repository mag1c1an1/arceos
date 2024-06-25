#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate libax;

#[cfg(not(target_arch = "aarch64"))]
use libax::{
    hv::{
        GuestPageTableTrait,
    },
};
use libax::hv::prelude::{init, vmcs_revision_id};
use libax::hv::vm::{boot_vm, arceos_config};

mod smp;

#[no_mangle]
fn main(hart_id: usize) {
    println!("Hello, hv!");
    println!("into main [hart_id: {}]", hart_id);
    init();
    let config = arceos_config();
    boot_vm(config);

    // init_virt_ipi(hart_id, libax::prelude::num_cpus());
    //
    // let mut p = PerCpu::<HyperCraftHalImpl>::new(hart_id);
    // p.hardware_enable().unwrap();
    //
    // load_bios_and_image();
    //
    //
    // let gpm = GUEST_PHY_MEMORY_SET.call_once(|| x64::setup_gpm().unwrap());
    //
    // info!("{:#x?}", gpm);
    //
    // let mut vcpu = p
    //     .create_vcpu(x64::BIOS_ENTRY, gpm.nest_page_table_root())
    //     .unwrap();
    //
    // println!("[{}] Running guest...", hart_id);
    // vcpu.run();

    // p.hardware_disable().unwrap();

    return;
}