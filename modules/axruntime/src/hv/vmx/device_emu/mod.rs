mod bundle;
mod debug_port;
mod dummy;
mod i8259_pic;
mod lapic;
mod pci;
mod pcip;
mod pit;
mod uart16550;

extern crate alloc;
use alloc::{sync::Arc, vec, vec::Vec};
use spin::Mutex;
use hypercraft::HyperResult;

use self::bundle::Bundle;
pub use self::lapic::VirtLocalApic;

pub trait PortIoDevice: Send + Sync {
    fn port_range(&self) -> core::ops::Range<u16>;
    fn read(&mut self, port: u16, access_size: u8) -> HyperResult<u32>;
    fn write(&mut self, port: u16, access_size: u8, value: u32) -> HyperResult;
}

pub struct VirtDeviceList {
    port_io_devices: Vec<Arc<Mutex<dyn PortIoDevice>>>,
}

impl VirtDeviceList {
    pub fn find_port_io_device(&self, port: u16) -> Option<&Arc<Mutex<dyn PortIoDevice>>> {
        self.port_io_devices
            .iter()
            .find(|dev| dev.lock().port_range().contains(&port))
    }
}

lazy_static::lazy_static! {
    static ref BUNDLE: Arc<Mutex<Bundle>> = Arc::new(Mutex::new(Bundle::new()));

    static ref VIRT_DEVICES : VirtDeviceList = VirtDeviceList {
        port_io_devices: vec![
            Arc::new(Mutex::new(uart16550::Uart16550::new(0x3f8))), // COM1
            Arc::new(Mutex::new(uart16550::Uart16550::new(0x2f8))), // COM2
            Arc::new(Mutex::new(uart16550::Uart16550::new(0x3e8))), // COM3
            Arc::new(Mutex::new(uart16550::Uart16550::new(0x2e8))), // COM4
            Arc::new(Mutex::new(i8259_pic::I8259Pic::new(0x20))), // PIC1
            Arc::new(Mutex::new(i8259_pic::I8259Pic::new(0xA0))), // PIC2
            Arc::new(Mutex::new(debug_port::DebugPort::new(0x80))), // Debug Port
            /*
                the complexity:
                - port 0x70 and 0x71 is for CMOS, but bit 7 of 0x70 is for NMI
                - port 0x40 ~ 0x43 is for PIT, but port 0x61 is also related
             */
            Arc::new(Mutex::new(Bundle::proxy_system_control_a(&BUNDLE))),
            Arc::new(Mutex::new(Bundle::proxy_system_control_b(&BUNDLE))),
            Arc::new(Mutex::new(Bundle::proxy_cmos(&BUNDLE))),
            Arc::new(Mutex::new(Bundle::proxy_pit(&BUNDLE))),
            Arc::new(Mutex::new(dummy::Dummy::new(0xf0, 2))), // 0xf0 and 0xf1 are ports about fpu
            Arc::new(Mutex::new(dummy::Dummy::new(0x3d4, 2))), // 0x3d4 and 0x3d5 are ports about vga
            Arc::new(Mutex::new(dummy::Dummy::new(0x87, 1))), // 0x87 is a port about dma
            Arc::new(Mutex::new(dummy::Dummy::new(0x60, 1))), // 0x60 and 0x64 are ports about ps/2 controller
            Arc::new(Mutex::new(dummy::Dummy::new(0x64, 1))), // 
            // Arc::new(Mutex::new(pci::PCIConfigurationSpace::new(0xcf8))),
            Arc::new(Mutex::new(pcip::PCIPassthrough::new(0xcf8))),
        ],
    };
}

pub fn all_virt_devices() -> &'static VirtDeviceList {
    &VIRT_DEVICES
}
