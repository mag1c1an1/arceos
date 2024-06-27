mod i8259_pic;
mod lapic;
mod uart16550;
mod shutdown;


extern crate alloc;

use alloc::{sync::Arc, vec, vec::Vec};
use spin::Mutex;
use hypercraft::{HyperError, HyperResult, VmxExitInfo};
use crate::hv::HyperCraftHalImpl;
use crate::hv::vcpu::VirtCpu;
use crate::hv::vmx::device_emu::i8259_pic::I8259Pic;
use crate::hv::vmx::device_emu::uart16550::Uart16550;
use crate::hv::vmx::VCpu;

pub use self::lapic::VirtLocalApic;

pub trait PioOps: Send + Sync {
    /// Port range.
    fn port_range(&self) -> core::ops::Range<u16>;
    /// Read operation
    fn read(&mut self, port: u16, access_size: u8) -> HyperResult<u32>;
    /// Write operation
    fn write(&mut self, port: u16, access_size: u8, value: u32) -> HyperResult;
}

pub struct VirtDeviceList {
    port_io_devices: Vec<Arc<dyn PioOps>>,
}

impl VirtDeviceList {
    pub fn find_port_io_device(&self, port: u16) -> Option<&Arc<dyn PioOps>> {
        self.port_io_devices
            .iter()
            .find(|dev| dev.port_range().contains(&port))
    }
}

pub struct X64VirtDevices {
    devices: DeviceList,
    pic: [Arc<Mutex<I8259Pic>>; 2],
}

impl X64VirtDevices {
    pub fn new() -> HyperResult<Self> {
        let pic: [Arc<Mutex<I8259Pic>>; 2] = [
            Arc::new(Mutex::new(I8259Pic::new(0x20))),
            Arc::new(Mutex::new(I8259Pic::new(0xA0))),
        ];
        let mut devices = DeviceList::new();

        let mut pmio_devices: Vec<Arc<Mutex<dyn PioOps>>> = vec![
            // 0x604
            Arc::new(Mutex::new(shutdown::Shutdown)),
            // These are all fully emulated consoles!!!
            // 0x3f8, 0x3f8 + 8
            Arc::new(Mutex::new(<Uart16550>::new(0x3f8))), // COM1
            // 0x2f8, 0x2f8 + 8
            Arc::new(Mutex::new(<Uart16550>::new(0x2f8))), // COM2
            // 0x3e8, 0x3e8 + 8
            Arc::new(Mutex::new(<Uart16550>::new(0x3e8))), // COM3
            // 0x2e8, 0x2e8 + 8
            Arc::new(Mutex::new(<Uart16550>::new(0x2e8))), // COM4
            // 0x20, 0x20 + 2
            pic[0].clone(), // PIC1
            // 0xa0, 0xa0 + 2
            pic[1].clone(), // PIC2
        ];
        devices.add_port_io_devices(&mut pmio_devices);
        Ok(Self { devices, pic })
    }
    pub fn handle_io_instruction(&mut self, vcpu: &mut VCpu, exit_info: &VmxExitInfo) -> HyperResult {
        self.devices.handle_io_instruction(vcpu, exit_info)
    }
}


pub struct DeviceList {
    port_io_devices: Vec<Arc<Mutex<dyn PioOps>>>,
}

impl DeviceList {
    pub fn new() -> Self {
        Self {
            port_io_devices: vec![],
        }
    }
    pub fn add_port_io_device(&mut self, device: Arc<Mutex<dyn PioOps>>) {
        self.port_io_devices.push(device)
    }

    pub fn add_port_io_devices(&mut self, devices: &mut Vec<Arc<Mutex<dyn PioOps>>>) {
        self.port_io_devices.append(devices)
    }

    pub fn find_port_io_device(&self, port: u16) -> Option<Arc<Mutex<dyn PioOps>>> {
        self.port_io_devices
            .iter()
            .find(|dev| dev.lock().port_range().contains(&port))
            .cloned()
        // todo
    }
    pub fn handle_io_instruction(&mut self, vcpu: &mut VCpu, exit_info: &VmxExitInfo) -> HyperResult {
        let io_info = vcpu.io_exit_info()?;
        if let Some(dev) = self.find_port_io_device(io_info.port) {
            Self::handle_io_instruction_to_device(vcpu, exit_info, dev)
        } else {
            Err(HyperError::Internal)
        }
    }

    fn handle_io_instruction_to_device(
        vcpu: &mut VCpu,
        exit_info: &VmxExitInfo,
        device: Arc<Mutex<dyn PioOps>>,
    ) -> HyperResult {
        let io_info = vcpu.io_exit_info().unwrap();
        trace!(
            "VM exit: I/O instruction @ {:#x}: {:#x?}",
            exit_info.guest_rip,
            io_info,
        );

        if io_info.is_string {
            error!("INS/OUTS instructions are not supported!");
            return Err(HyperError::NotSupported);
        }
        if io_info.is_repeat {
            error!("REP prefixed I/O instructions are not supported!");
            return Err(HyperError::NotSupported);
        }
        if io_info.is_in {
            let value = device.lock().read(io_info.port, io_info.access_size)?;
            let rax = &mut vcpu.regs_mut().rax;
            // SDM Vol. 1, Section 3.4.1.1:
            // * 32-bit operands generate a 32-bit result, zero-extended to a 64-bit result in the
            //   destination general-purpose register.
            // * 8-bit and 16-bit operands generate an 8-bit or 16-bit result. The upper 56 bits or
            //   48 bits (respectively) of the destination general-purpose register are not modified
            //   by the operation.
            match io_info.access_size {
                1 => *rax = (*rax & !0xff) | (value & 0xff) as u64,
                2 => *rax = (*rax & !0xffff) | (value & 0xffff) as u64,
                4 => *rax = value as u64,
                _ => unreachable!(),
            }
        } else {
            let rax = vcpu.regs().rax;
            let value = match io_info.access_size {
                1 => rax & 0xff,
                2 => rax & 0xffff,
                4 => rax,
                _ => unreachable!(),
            } as u32;
            device
                .lock()
                .write(io_info.port, io_info.access_size, value)?;
        }
        vcpu.advance_rip(exit_info.exit_instruction_length as _)?;
        Ok(())
    }
}



