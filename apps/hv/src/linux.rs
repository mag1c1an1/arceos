use axvm;
use axprocess;

use axvm::LinuxContext;

#[cfg(feature = "type1_5")]
pub fn boot_linux(hart_id: usize, linux_context: &LinuxContext) {
    axvm::config_boot_linux(hart_id, linux_context);
}

#[cfg(not(feature = "type1_5"))]
pub fn boot_linux(hart_id: usize) {
    axvm::config_boot_linux(hart_id);

    loop {
        // if unsafe { axprocess::wait_pid(now_process_id, &mut exit_code as *mut i32) }.is_ok() {
        //     break Some(exit_code);
        // }

        axprocess::yield_now_task();
    };
}