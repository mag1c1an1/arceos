//! Emulated Local APIC. (SDM Vol. 3A, Chapter 10)

#![allow(dead_code)]

use alloc::vec;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;
use spin::once::Once;
use x2apic::lapic::IpiAllShorthand::AllExcludingSelf;
use axconfig::SMP;
use crate::hv::vmx::smp::{DeliveryMode, Icr};
use hypercraft::{HyperError, HyperResult, VCpu as HVCpu};
use hypercraft::smp::{broadcast_message, Message, send_message, Signal};
use crate::hv::vmx::HV_VIRT_IPI;


pub static BOOT_VEC: AtomicBool = AtomicBool::new(false);


// fn init_boot_vec(smp: usize) {
//     BOOT_VEC.call_once(|| {
//         let mut v = vec![];
//         for i in 0..smp {
//             v.push(false);
//         }
//         Mutex::new(v)
//     });
// }


type VCpu = HVCpu<crate::hv::HyperCraftHalImpl>;

/// ID register.
const APICID: u32 = 0x2;
/// Version register.
const VERSION: u32 = 0x3;
/// EOI register.
const EOI: u32 = 0xB;
/// Logical Destination Register.
const LDR: u32 = 0xD;
/// Spurious Interrupt Vector register.
const SIVR: u32 = 0xF;
/// Interrupt Command register.
const ICR: u32 = 0x30;
/// LVT Timer Interrupt register.
const LVT_TIMER: u32 = 0x32;
/// LVT Thermal Sensor Interrupt register.
const LVT_THERMAL: u32 = 0x33;
/// LVT Performance Monitor register.
const LVT_PMI: u32 = 0x34;
/// LVT LINT0 register.
const LVT_LINT0: u32 = 0x35;
/// LVT LINT1 register.
const LVT_LINT1: u32 = 0x36;
/// LVT Error register.
const LVT_ERR: u32 = 0x37;
///  Initial Count register.
const INIT_COUNT: u32 = 0x38;
/// Current Count register.
const CUR_COUNT: u32 = 0x39;
/// Divide Configuration register.
const DIV_CONF: u32 = 0x3E;

pub struct VirtLocalApic;

impl VirtLocalApic {
    pub const fn msr_range() -> core::ops::Range<u32> {
        0x800..0x840
    }

    pub fn rdmsr(VCpu: &mut VCpu, msr: u32) -> HyperResult<u64> {
        Self::read(VCpu, msr - 0x800)
    }

    pub fn wrmsr(VCpu: &mut VCpu, msr: u32, value: u64) -> HyperResult {
        Self::write(VCpu, msr - 0x800, value)
    }
}

impl VirtLocalApic {
    fn read(VCpu: &mut VCpu, offset: u32) -> HyperResult<u64> {
        let apic_timer = VCpu.apic_timer_mut();
        match offset {
            SIVR => Ok(0x1ff), // SDM Vol. 3A, Section 10.9, Figure 10-23 (with Software Enable bit)
            LVT_THERMAL | LVT_PMI | LVT_LINT0 | LVT_LINT1 | LVT_ERR => {
                Ok(0x1_0000) // SDM Vol. 3A, Section 10.5.1, Figure 10-8 (with Mask bit)
            }
            LVT_TIMER => Ok(apic_timer.lvt_timer() as u64),
            INIT_COUNT => Ok(apic_timer.initial_count() as u64),
            DIV_CONF => Ok(apic_timer.divide() as u64),
            CUR_COUNT => Ok(apic_timer.current_counter() as u64),
            _ => Err(HyperError::NotSupported),
        }
    }

    fn write(VCpu: &mut VCpu, offset: u32, value: u64) -> HyperResult {
        if offset != ICR && (value >> 32) != 0 {
            return Err(HyperError::InvalidParam); // all registers except ICR are 32-bits
        }
        let apic_timer = VCpu.apic_timer_mut();
        match offset {
            EOI => {
                if value != 0 {
                    Err(HyperError::InvalidParam) // write a non-zero value causes #GP
                } else {
                    Ok(())
                }
            }
            SIVR | LVT_THERMAL | LVT_PMI | LVT_LINT0 | LVT_LINT1 | LVT_ERR => {
                Ok(()) // ignore these register writes
            }
            ICR => send_ipi(value), // FIXME:
            LVT_TIMER => apic_timer.set_lvt_timer(value as u32),
            INIT_COUNT => apic_timer.set_initial_count(value as u32),
            DIV_CONF => apic_timer.set_divide(value as u32),
            _ => Err(HyperError::NotSupported),
        }
    }
}

fn send_ipi(value: u64) -> HyperResult {
    unsafe {
        let icr = Icr(value);
        debug!("icr: {:?}", icr);
        debug!("in icr value:  {:X}H", value);
        let mode = DeliveryMode::try_from(icr.delivery_mode()).unwrap();
        match mode {
            DeliveryMode::Fixed => todo!(),
            DeliveryMode::LowPriority => todo!(),
            DeliveryMode::SMI => todo!(),
            DeliveryMode::NMI => todo!(),
            DeliveryMode::INIT => {
                debug!("ignore INIT IPI");
                Ok(())
            }
            DeliveryMode::StartUp => {
                debug!("send start up ipi");
                /// FIXME
                /// just send a ipi to others
                let msg = Message {
                    dest: 0,
                    signal: Signal::Start,
                    args: vec![icr.vector() as usize],
                };
                if !BOOT_VEC.load(Ordering::Relaxed) {
                    broadcast_message(msg);
                    axhal::mp::send_ipi_all(HV_VIRT_IPI as u8, AllExcludingSelf);
                    BOOT_VEC.store(true,Ordering::Relaxed);
                }
                Ok(())
            }
        }
    }
}
