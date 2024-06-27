use core::ops::Range;
use hypercraft::{HyperError, HyperResult};
use crate::hv::vmx::device_emu::PioOps;

pub struct Shutdown;

impl PioOps for Shutdown {
    fn port_range(&self) -> Range<u16> {
        0x604u16..0x605u16
    }

    fn read(&mut self, port: u16, access_size: u8) -> HyperResult<u32> {
        todo!()
    }

    fn write(&mut self, port: u16, access_size: u8, value: u32) -> HyperResult {
        Err(HyperError::Shutdown)
    }
}
