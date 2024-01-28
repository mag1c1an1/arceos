//! Syscall Forwarding.

mod allocator;
mod queue;

pub mod syscall_proxy;

/// Configuration for syscall forward in arceos.
///
/// Todo: move these to `axconfig`.
mod cfg {
    pub const SYSCALL_IPI_IRQ_NUM: usize = 13;

    pub const SYSCALL_DATA_BUF_SIZE: usize = axconfig::SYSCALL_DATA_BUF_SIZE;
    pub const SYSCALL_QUEUE_BUF_SIZE: usize = axconfig::SYSCALL_QUEUE_BUF_SIZE;

    pub const SYSCALL_DATA_BUF_PADDR: usize = axconfig::PHYS_MEMORY_END;
    pub const SYSCALL_QUEUE_BUF_PADDR: usize = SYSCALL_DATA_BUF_PADDR + SYSCALL_DATA_BUF_SIZE;
}

pub fn notify() {
    axhal::irq::send_ipi(cfg::SYSCALL_IPI_IRQ_NUM);
}

pub fn init() {
    queue::init();
    allocator::init();
}
