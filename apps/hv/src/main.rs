#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate libax;

use axvm::LinuxContext;

mod linux;

#[cfg(feature = "type1_5")]
#[no_mangle]
fn main(cpu_id: u32, linux_context: &LinuxContext) {
    info!("Hello, hv!");
    info!("Currently Linux inside VM is on Core {}", cpu_id);
    linux::boot_linux(cpu_id as usize, linux_context);

    panic!("Should never return!!!");
}

#[cfg(not(feature = "type1_5"))]
#[no_mangle]
fn main() {
    println!("Hello, hv!");
    println!("Currently Linux inside VM is pinned on Core 0");
    // linux::boot_linux(0);

    loop {
        libax::thread::sleep(libax::time::Duration::from_secs(1));
        println!("main tick");
    }
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub fn main_secondary(hart_id: usize) {
    println!("Hello, processs on core {}!", hart_id);

    // process::hello();

    // loop {
    //     libax::thread::sleep(libax::time::Duration::from_secs(1));
    //     println!("secondary tick");
    // }
}
