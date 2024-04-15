//! x86 page table entries on 64-bit paging.
mod epte;
mod pte;

pub use epte::EPTEntry;
pub use pte::{PTF, X64PTE};
