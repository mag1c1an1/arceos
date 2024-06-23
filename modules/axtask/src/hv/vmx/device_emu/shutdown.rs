use core::ops::Range;
use hypercraft::{HyperError, HyperResult};
use crate::hv::vmx::device_emu::PortIoDevice;

pub struct Shutdown;

impl PortIoDevice for Shutdown {
    fn port_range(&self) -> Range<u16> {
        0x604u16..0x605u16
    }

    fn read(&self, port: u16, access_size: u8) -> HyperResult<u32> {
        todo!()
    }

    fn write(&self, port: u16, access_size: u8, value: u32) -> HyperResult {
        Err(HyperError::Shutdown)
    }
}
