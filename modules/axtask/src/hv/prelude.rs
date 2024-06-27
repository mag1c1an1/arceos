use axconfig::SMP;
use crate::hv::{pcpu, vm, notify};

pub fn init() {
    vm::init();
    notify::init_mgs_lists(SMP);
}

pub use pcpu::vmcs_revision_id;