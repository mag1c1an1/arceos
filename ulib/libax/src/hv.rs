//! Hypervisor related functions

pub use axhal::mem::{phys_to_virt, PhysAddr, virt_to_phys};
pub use axruntime::GuestPageTable;
pub use axruntime::HyperCraftHalImpl;
pub use hypercraft::{PerCpu, VCpu, VM, VmCpus};
#[cfg(not(target_arch = "aarch64"))]
pub use hypercraft::{GuestPhysAddr, GuestVirtAddr, HostPhysAddr, HostVirtAddr, HyperCallMsg, VmExitInfo};
pub use hypercraft::GuestPageTableTrait;
pub use hypercraft::HyperCraftHal;
pub use hypercraft::HyperError as Error;
pub use hypercraft::HyperResult as Result;
pub use hypercraft::smp::*;
