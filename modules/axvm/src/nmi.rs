use spin::Mutex;

use axconfig::SMP;

extern crate alloc;
use alloc::vec::Vec;

const MAX_CPUS: usize = 8;

pub static CPU_NMI_LIST: [Mutex<NmiMsgQueue>; SMP] = [Mutex::new(NmiMsgQueue::default()); SMP];

#[derive(Default)]
pub struct NmiMsgQueue {
    pub msg_queue: Vec<NmiMessage>,
}

#[derive(Copy, Clone, Debug)]
pub enum NmiMessage {
    // vm_id
    BootVm(usize),
}

impl NmiMsgQueue {
    // pub fn default() -> NmiMsgQueue {
    //     NmiMsgQueue {
    //         msg_queue: Vec::new(),
    //     }
    // }

    pub fn push(&mut self, ipi_msg: NmiMessage) {
        self.msg_queue.push(ipi_msg);
    }

    pub fn pop(&mut self) -> Option<NmiMessage> {
        self.msg_queue.pop()
    }
}

pub fn nmi_send_msg(target_cpu_id: usize, msg: NmiMessage) {
    CPU_NMI_LIST[target_cpu_id].lock().msg_queue.push(msg);
    debug!(
        "cpu_int_list {:?}",
        CPU_NMI_LIST[target_cpu_id].lock().msg_queue
    );
    // send ipi to target core
    axhal::irq::send_nmi_to(target_cpu_id);
}
