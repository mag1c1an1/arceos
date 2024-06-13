// use libax::hv::{HyperCraftHalImpl, PerCpu};

use alloc::vec::Vec;
use spin::Mutex;
use libax::hv::{HyperCraftHalImpl, PerCpu, receive_message};

use spin::once::Once;
use crate::x64;



#[no_mangle]
fn hv_main_secondary(hart_id: usize) {
    println!("[hart_id: {}] enter main secondary", hart_id);
    // should hlt
    // #[cfg(target_arch = "x86_64")]
    // {
    //     let mut p = PerCpu::<HyperCraftHalImpl>::new(hart_id);
    //     p.hardware_enable().unwrap();

    //     let gpm = x64::setup_gpm().unwrap();
    //     info!("{:#x?}", gpm);

    //     let mut vcpu = p
    //         .create_vcpu(x64::BIOS_ENTRY, gpm.nest_page_table_root())
    //         .unwrap();

    //     println!("Running guest...");
    //     vcpu.run();
    //     p.hardware_disable().unwrap();

    //     return;
    // }
}


#[no_mangle]
fn hv_virt_ipi_handler(hart_id: usize) {
    let msg = receive_message(hart_id);
    match msg.signal {
        libax::hv::Signal::Start => {
            let start_addr = msg.args[0];
            ap_start(hart_id, start_addr);
        }
    }
}


fn ap_start(hart_id: usize, start_addr: usize) {
    let start_addr = start_addr << 12;
    println!("[{}] hv ap start 0x{:x}", hart_id, start_addr);

    let mut p = PerCpu::<HyperCraftHalImpl>::new(hart_id);
    p.hardware_enable().unwrap();

    let gpm = x64::setup_gpm().unwrap();
    info!("{:#x?}", gpm);

    let mut vcpu = p
        .create_vcpu(start_addr, gpm.nest_page_table_root())
        .unwrap();

    println!("Running guest...");
    vcpu.run();

    p.hardware_disable().unwrap();
    return;
}