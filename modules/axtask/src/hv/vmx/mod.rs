mod device_emu;
pub mod smp;

use alloc::sync::Arc;
use axhal::cpu::this_cpu_id;
use device_emu::VirtLocalApic;
use hypercraft::{HyperError, HyperResult, VCpu as HVCpu, VmxExitInfo, VmxExitReason};
use crate::hv::vcpu::VirtCpu;
use crate::on_timer_tick;

pub use device_emu::X64VirtDevices;

type VCpu = HVCpu<super::HyperCraftHalImpl>;

const VM_EXIT_INSTR_LEN_CPUID: u8 = 2;
const VM_EXIT_INSTR_LEN_RDMSR: u8 = 2;
const VM_EXIT_INSTR_LEN_WRMSR: u8 = 2;
const VM_EXIT_INSTR_LEN_VMCALL: u8 = 3;

pub fn handle_external_interrupt(vcpu: &VirtCpu) -> HyperResult {
    #[cfg(feature = "irq")]
    {
        let int_info = vcpu.vmx_vcpu_mut().interrupt_exit_info()?;
        trace!("VM-exit: external interrupt: {:#x?}", int_info);
        assert!(int_info.valid);
        error!("{} handle external irq {}",vcpu,int_info.vector as usize);
        axhal::irq::dispatch_irq(int_info.vector as usize);
        Ok(())
    }
    #[cfg(not(feature = "irq"))]
    {
        panic!("cannot handle EXTERNAL_INTERRUPT vmexit because \"irq\" is not enabled")
    }
}

fn handle_cpuid(vcpu: &Arc<VirtCpu>) -> HyperResult {
    use raw_cpuid::{cpuid, CpuIdResult};

    const LEAF_FEATURE_INFO: u32 = 0x1;
    const LEAF_HYPERVISOR_INFO: u32 = 0x4000_0000;
    const LEAF_HYPERVISOR_FEATURE: u32 = 0x4000_0001;
    const VENDOR_STR: &[u8; 12] = b"RVMRVMRVMRVM";

    let vendor_regs = unsafe { &*(VENDOR_STR.as_ptr() as *const [u32; 3]) };
    let regs = vcpu.vmx_vcpu_mut().regs_mut();
    let function = regs.rax as u32;
    let res = match function {
        LEAF_FEATURE_INFO => {
            error!("vmx get cpu id");
            const FEATURE_VMX: u32 = 1 << 5;
            const FEATURE_HYPERVISOR: u32 = 1 << 31;
            let mut res = cpuid!(regs.rax, regs.rcx);
            res.ecx &= !FEATURE_VMX;
            res.ecx |= FEATURE_HYPERVISOR;
            res
        }
        LEAF_HYPERVISOR_INFO => CpuIdResult {
            eax: LEAF_HYPERVISOR_FEATURE,
            ebx: vendor_regs[0],
            ecx: vendor_regs[1],
            edx: vendor_regs[2],
        },
        LEAF_HYPERVISOR_FEATURE => CpuIdResult {
            eax: 0,
            ebx: 0,
            ecx: 0,
            edx: 0,
        },
        _ => {
            debug!("host [{}] passthrough cpuid", this_cpu_id());
            cpuid!(regs.rax, regs.rcx)
        }
    };

    debug!(
        "VM exit: CPUID({:#x}, {:#x}): {:?}",
        regs.rax, regs.rcx, res
    );
    regs.rax = res.eax as _;
    regs.rbx = res.ebx as _;
    regs.rcx = res.ecx as _;
    regs.rdx = res.edx as _;
    vcpu.vmx_vcpu_mut().advance_rip(VM_EXIT_INSTR_LEN_CPUID)?;
    Ok(())
}

pub fn handle_msr_read(vcpu: &VirtCpu) -> HyperResult {
    let msr = vcpu.vmx_vcpu_mut().regs().rcx as u32;

    use x86::msr::*;
    let res = if msr == IA32_APIC_BASE {
        let mut apic_base = unsafe { rdmsr(IA32_APIC_BASE) };
        apic_base |= 1 << 11 | 1 << 10; // enable xAPIC and x2APIC
        Ok(apic_base)
    } else if VirtLocalApic::msr_range().contains(&msr) {
        VirtLocalApic::rdmsr(vcpu, msr)
    } else {
        Err(HyperError::NotSupported)
    };

    if let Ok(value) = res {
        trace!("VM exit: RDMSR({:#x}) -> {:#x}", msr, value);
        vcpu.vmx_vcpu_mut().regs_mut().rax = value & 0xffff_ffff;
        vcpu.vmx_vcpu_mut().regs_mut().rdx = value >> 32;
    } else {
        panic!("Failed to handle RDMSR({:#x}): {:?}", msr, res);
    }
    vcpu.vmx_vcpu_mut().advance_rip(VM_EXIT_INSTR_LEN_RDMSR)?;
    Ok(())
}

pub fn handle_msr_write(vcpu: &VirtCpu) -> HyperResult {
    let msr = vcpu.vmx_vcpu_mut().regs().rcx as u32;
    let value = (vcpu.vmx_vcpu_mut().regs().rax & 0xffff_ffff) | (vcpu.vmx_vcpu_mut().regs().rdx << 32);
    trace!("VM exit: WRMSR({:#x}) <- {:#x}", msr, value);

    use x86::msr::*;
    let res = if msr == IA32_APIC_BASE {
        Ok(()) // ignore
    } else if VirtLocalApic::msr_range().contains(&msr) {
        VirtLocalApic::wrmsr(vcpu, msr, value)
    } else {
        Err(HyperError::NotSupported)
    };

    if res.is_err() {
        panic!(
            "Failed to handle WRMSR({:#x}) <- {:#x}: {:?}",
            msr, value, res
        );
    }
    vcpu.vmx_vcpu_mut().advance_rip(VM_EXIT_INSTR_LEN_WRMSR)?;
    Ok(())
}
