use crate::Result;

use axhal::irq::{dispatch_irq, set_enable};

pub(crate) fn dispatch_host_irq(vector: usize) -> Result {
    dispatch_irq(vector);
    Ok(())
}
