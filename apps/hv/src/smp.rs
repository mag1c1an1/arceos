use libax::hv::{HyperCraftHalImpl, PerCpu, receive_message};
use crate::{GUEST_PHY_MEMORY_SET, x64};


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
        .create_vcpu(start_addr, GUEST_PHY_MEMORY_SET.get().unwrap().nest_page_table_root())
        .unwrap();

    println!("[{}] Running guest...", hart_id);
    vcpu.run();

    p.hardware_disable().unwrap();
    return;
}