#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate libax;

mod linux;

#[cfg(feature = "type1_5")]
#[no_mangle]
fn main(core_id: u32) {
    info!("Hello, hv!");
    info!("Currently Linux inside VM is on Core {}", core_id);
    linux::boot_linux();

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
