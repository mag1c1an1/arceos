//! Emulated Intel 8259 Programmable Interrupt Controller. (ref: https://wiki.osdev.org/8259_PIC)

use bit_field::BitField;
use hypercraft::{HyperError, HyperResult};
use crate::hv::vmx::device_emu::PioOps;

pub struct I8259Pic {
    port_base: u16,
    icw1: u8,
    offset: u8,
    icw3: u8,
    icw4: u8,
    icw_written: u8,
    icw_left: bool,
    mask: u8,
}

impl PioOps for I8259Pic {
    fn port_range(&self) -> core::ops::Range<u16> {
        self.port_base..self.port_base + 2
    }

    fn read(&mut self, port: u16, _access_size: u8) -> HyperResult<u32> {
        // let pic_name = if self.port_base == 0x20 {
        //     "Primary PIC"
        // } else if self.port_base == 0xa0 {
        //     "Secondary PIC"
        // } else {
        //     "Unknown"
        // };
        // debug!("reading from {pic_name} port {port:#x} size {_access_size:#x}");
        match port - self.port_base {
            1 => Ok(self.mask as u32),
            _ => Err(HyperError::NotSupported),
        }
    }

    fn write(&mut self, port: u16, _access_size: u8, value: u32) -> HyperResult {
        // let pic_name = if self.port_base == 0x20 {
        //     "Primary PIC"
        // } else if self.port_base == 0xa0 {
        //     "Secondary PIC"
        // } else {
        //     "Unknown"
        // };

        // debug!("writing to {pic_name} port {port:#x}: {value:#x} size {_access_size:#x}");

        let value = value as u8;
        match port - self.port_base {
            0 => {
                if value.get_bit(4) {
                    self.icw1 = value;
                    self.icw_left = true;
                    self.icw_written = 1;
                } else {
                    // debug!("pit ocw ignored");
                }
            }
            1 => {
                if !self.icw_left {
                    // debug!("set PIC mask {value:#x}");
                    self.mask = value;
                } else {
                    match self.icw_written {
                        1 => self.offset = value,
                        2 => self.icw3 = value,
                        3 => self.icw4 = value,
                        _ => return Err(HyperError::BadState),
                    }

                    if self.icw_written == 3 || (self.icw_written == 2 && !self.icw1.get_bit(0)) {
                        self.icw_left = false;
                        self.icw_written = 0;
                    } else {
                        self.icw_written += 1;
                    }
                }
            }
            _ => return Err(HyperError::InvalidParam),
        }

        Ok(()) // ignore write
    }
}

impl I8259Pic {
    pub const fn new(port_base: u16) -> Self {
        Self {
            port_base,
            icw1: 0,
            offset: 0,
            icw3: 0,
            icw4: 0,
            icw_left: false,
            icw_written: 0,
            mask: 0,
        }
    }

    pub const fn mask(&self) -> u8 {
        self.mask
    }
}
