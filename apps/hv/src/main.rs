#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate libax;

use core::time::Duration;
#[cfg(not(target_arch = "aarch64"))]
use libax::{
    hv::{
        GuestPageTableTrait,
    },
};
use libax::hv::vm::{create_vm, nimbos_config};
use libax::thread::sleep;

// #[cfg(target_arch = "x86_64")]
// mod x64;

mod smp;

#[no_mangle]
fn main(hart_id: usize) {
    println!("Hello, hv!");
    println!("into main [hart_id: {}]", hart_id);
    // let y = libax::thread::spawn(|| {
    //     loop {
    //         println!("xxx");
    //         sleep(Duration::from_millis(100));
    //     }
    // });

    let x = libax::thread::spawn(|| {
        let config = nimbos_config();
        create_vm(config).unwrap()
    });
    let id = x.join().unwrap();

    println!("id is {}", id);

    // y.join().unwrap();
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