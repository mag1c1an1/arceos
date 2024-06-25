use crate::hv::{pcpu, vm};

pub fn init() {
    vm::init();
}

pub use pcpu::vmcs_revision_id;