//! Emulated Local APIC. (SDM Vol. 3A, Chapter 10)

#![allow(dead_code)]
use hypercraft::{VCpu as HVCpu, HyperResult, HyperError};

type VCpu = HVCpu<crate::hv::HyperCraftHalImpl>;

/// ID register.
const APICID: u32 = 0x2;
/// Version register.
const VERSION: u32 = 0x3;
/// Task priority register.
const TPR: u32 = 0x8;
/// EOI register.
const EOI: u32 = 0xB;
/// Logical Destination Register.
const LDR: u32 = 0xD;
/// Spurious Interrupt Vector register.
const SIVR: u32 = 0xF;
/// In-Service Register 0
const ISR0: u32 = 0x10;
/// In-Service Register 1
const ISR1: u32 = 0x11;
/// In-Service Register 2
const ISR2: u32 = 0x12;
/// In-Service Register 3
const ISR3: u32 = 0x13;
/// In-Service Register 4
const ISR4: u32 = 0x14;
/// In-Service Register 5
const ISR5: u32 = 0x15;
/// In-Service Register 6
const ISR6: u32 = 0x16;
/// In-Service Register 7
const ISR7: u32 = 0x17;
/// Interrupt Request Register 0
const IRR0: u32 = 0x20;
/// Interrupt Request Register 1
const IRR1: u32 = 0x21;
/// Interrupt Request Register 2
const IRR2: u32 = 0x22;
/// Interrupt Request Register 3
const IRR3: u32 = 0x23;
/// Interrupt Request Register 4
const IRR4: u32 = 0x24;
/// Interrupt Request Register 5
const IRR5: u32 = 0x25;
/// Interrupt Request Register 6
const IRR6: u32 = 0x26;
/// Interrupt Request Register 7
const IRR7: u32 = 0x27;
/// Error Status Register.
const ESR: u32 = 0x28;
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
            },
            IRR0 ..= IRR7 => Ok(0),
            ISR0 ..= ISR7 => Ok(0),
            LVT_TIMER => Ok(apic_timer.lvt_timer() as u64),
            INIT_COUNT => Ok(apic_timer.initial_count() as u64),
            DIV_CONF => Ok(apic_timer.divide() as u64),
            CUR_COUNT => Ok(apic_timer.current_counter() as u64),
            LDR => Ok(0),
            TPR => Ok(apic_timer.tpr() as u64),
            VERSION => Ok(0b0000000_0_00000000_00000110_00010101), // Suppress EOI-broadcasts: false, Max LVT Entry: 6, Version: 0x15
            ESR => Ok(0),
            APICID => Ok(0),
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
            LVT_TIMER => apic_timer.set_lvt_timer(value as u32),
            INIT_COUNT => apic_timer.set_initial_count(value as u32),
            DIV_CONF => apic_timer.set_divide(value as u32),
            TPR => Ok(apic_timer.set_tpr(value as u32)),
            ESR => if value == 0 { Ok(()) } else { Err(HyperError::InvalidParam) },
            _ => Err(HyperError::NotSupported),
        }
    }
}
