use libax::hv::{HyperCraftHalImpl, PerCpu};

use crate::x64;

#[no_mangle]
fn hv_main_secondary(hart_id: usize) {
    println!("[hart_id: {}] enter main secondary");
    // should hlt
    #[cfg(target_arch = "x86_64")] {
        let mut p = PerCpu::<HyperCraftHalImpl>::new(hart_id);
        p.hardware_enable().unwrap();

        let gpm = x64::setup_gpm().unwrap();
        info!("{:#x?}", gpm);

        let mut vcpu = p
            .create_vcpu(x64::BIOS_ENTRY, gpm.nest_page_table_root())
            .unwrap();

        println!("Running guest...");
        vcpu.run();
        p.hardware_disable().unwrap();

        return;
    }
}