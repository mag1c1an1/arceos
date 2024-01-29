use core::sync::atomic::{AtomicBool, Ordering};
use axhal::arch::{disable_irqs, irqs_enabled, enable_irqs};


pub fn spin_lock_irqsave(lock: &AtomicBool) -> bool {
    let irq_enabled_before = irqs_enabled();
    disable_irqs();
    while lock
        .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        while lock.load(Ordering::Relaxed) {
            core::hint::spin_loop();
        }
    }
    irq_enabled_before
}

pub fn spin_trylock_irqsave(lock: &AtomicBool) -> Option<bool> {
    let irq_enabled_before = irqs_enabled();
    disable_irqs();
    if lock
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_ok()
    {
        Some(irq_enabled_before)
    } else {
        if irq_enabled_before {
            enable_irqs();
        }
        None
    }
}

pub fn spin_unlock_irqrestore(lock: &AtomicBool, irq_enabled_before: bool) {
    lock.store(false, Ordering::Release);
    if irq_enabled_before {
        enable_irqs();
    }
}