/// Temporar module to boot Linux as a guest VM.
///
/// To be removed...
// use hypercraft::GuestPageTableTrait;
use hypercraft::{PerCpu, VCpu, VmCpus, VM};
#[cfg(feature = "type1_5")]
use hypercraft::LinuxContext;
#[cfg(feature = "type1_5")]
use crate::config::{root_gpm, init_gpm};

#[cfg(target_arch = "x86_64")]
use super::device::{self, X64VcpuDevices, X64VmDevices};
use super::arch::new_vcpu;
use axhal::hv::HyperCraftHalImpl;

use core::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
// use super::type1_5::cell;
static INIT_GPM_OK: AtomicU32 = AtomicU32::new(0);
static INITED_CPUS: AtomicUsize = AtomicUsize::new(0);

#[cfg(feature = "type1_5")]
pub fn config_boot_linux(hart_id: usize, linux_context: &LinuxContext) {
    info!("CPU{} into config_boot_linux", hart_id);
    crate::arch::cpu_hv_hardware_enable(hart_id, linux_context);
    info!("CPU{} hardware_enable done", hart_id);
    if hart_id == 0 {
        super::config::init_gpm();
        INIT_GPM_OK.store(1, Ordering::Release);
    }else {
        while INIT_GPM_OK.load(Ordering::Acquire) < 1 {
            core::hint::spin_loop();
        }
    }
    info!("CPU{} after init_gpm", hart_id);
    
    // let gpm = super::config::setup_gpm().unwrap();
    let gpm = super::config::root_gpm();
    debug!("CPU{} type 1.5 gpm: {:#x?}", hart_id, gpm);
    let vcpu = new_vcpu(hart_id, crate::arch::cpu_vmcs_revision_id(), gpm.nest_page_table_root(), &linux_context).unwrap();
    let mut vcpus = VmCpus::<HyperCraftHalImpl, X64VcpuDevices<HyperCraftHalImpl>>::new();
    info!("CPU{} add vcpu to vm...", hart_id);
    vcpus.add_vcpu(vcpu).expect("add vcpu failed");
    let mut vm = VM::<
        HyperCraftHalImpl,
        X64VcpuDevices<HyperCraftHalImpl>,
        X64VmDevices<HyperCraftHalImpl>,
    >::new(vcpus);
    // The bind_vcpu method should be decoupled with vm struct.
    vm.bind_vcpu(hart_id).expect("bind vcpu failed");
    
    INITED_CPUS.fetch_add(1, Ordering::SeqCst);
    while INITED_CPUS.load(Ordering::Acquire) < axconfig::SMP {
        core::hint::spin_loop();
    }

    debug!("CPU{} before run vcpu", hart_id);
    info!("{:?}", vm.run_type15_vcpu(hart_id, &linux_context));

    // disable hardware virtualization todo
}

#[cfg(not(feature = "type1_5"))]
pub fn config_boot_linux(hart_id: usize) {
    info!("into main {}", hart_id);

    // Fix: this function shoule be moved to somewhere like vm_entry.
    crate::arch::cpu_hv_hardware_enable(hart_id);

    // Alloc guest memory set.
    // Fix: this should be stored inside VM structure.
    let gpm = super::config::setup_gpm(hart_id).unwrap();
    let npt = gpm.nest_page_table_root();
    info!("{:#x?}", gpm);

    // Main scheduling item, managed by `axtask`
    let vcpu = VCpu::new(0, crate::arch::cpu_vmcs_revision_id(), 0x7c00, npt).unwrap();

    let mut vcpus = VmCpus::<HyperCraftHalImpl, X64VcpuDevices<HyperCraftHalImpl>>::new();
    vcpus.add_vcpu(vcpu).expect("add vcpu failed");

    let mut vm = VM::<
        HyperCraftHalImpl,
        X64VcpuDevices<HyperCraftHalImpl>,
        X64VmDevices<HyperCraftHalImpl>,
    >::new(vcpus);
	
	// The bind_vcpu method should be decoupled with vm struct.
    vm.bind_vcpu(0).expect("bind vcpu failed");

    if hart_id == 0 {
        let (_, dev) = vm.get_vcpu_and_device(0).unwrap();
        *(dev.console.lock().backend()) = device::device_emu::MultiplexConsoleBackend::Primary;

        for v in 0..256 {
            crate::irq::set_host_irq_enabled(v, true);
        }
    }

    info!("Running guest...");
    info!("{:?}", vm.run_vcpu(0));

    crate::arch::cpu_hv_hardware_disable();

    panic!("done");
}
