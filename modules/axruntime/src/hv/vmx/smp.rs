use num_enum::{IntoPrimitive, TryFromPrimitive};

#[repr(u64)]
#[derive(Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum DeliveryMode {
    Fixed = 0b000,
    LowPriority = 0b001,
    SMI = 0b010,
    NMI = 0b100,
    INIT = 0b101,
    StartUp = 0b110,
}

#[repr(u64)]
#[derive(Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum DestinationMode {
    Physical = 0,
    Logical = 1,
}

#[repr(u64)]
#[derive(Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum DeliveryStatus {
    Idle = 0,
    SendPending = 1,
}

#[repr(u64)]
#[derive(Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum Level {
    DeAssert = 0,
    Assert = 1,
}

#[repr(u64)]
#[derive(Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum TriggerMode {
    Edge = 0,
    Level = 1,
}

bitfield::bitfield! {
    pub struct Icr(u64);
    impl Debug;
    pub vector, set_vector: 7, 0;
    pub delivery_mode, set_delivery_mode: 10, 8;
    pub destination_mode, set_destination_mode: 11, 11;
    pub delivery_status, _: 12, 12;
    pub level, set_level: 14, 14;
    pub trigger_mode, set_trigger_mode: 15, 15;
    pub destination_shorthand, set_destination_shorthand: 19, 18;
    pub destination_field, set_destination_field: 63,56;
    pub low, set_low: 31, 0;
    pub high, set_high: 63, 32;
}

impl Default for Icr {
    fn default() -> Self {
        Icr(0)
    }
}

// /// sender
// pub struct VirtIPISender {
//     bsp: usize,
//     start_sended: Vec<usize>,
// }
//
// impl VirtIPISender {
//
// }
//
