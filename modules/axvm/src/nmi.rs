use alloc::collections::LinkedList;

use spin::Mutex;

use axconfig::SMP;

const MAX_CPUS: usize = 8;

const PER_CPU_NMI_MSG_QUEUE: Mutex<NmiMsgQueue> = Mutex::new(NmiMsgQueue::new());
pub static CPU_NMI_LIST: [Mutex<NmiMsgQueue>; SMP] = [PER_CPU_NMI_MSG_QUEUE; SMP];

pub struct NmiMsgQueue {
    msg_queue: LinkedList<NmiMessage>,
}

#[derive(Copy, Clone, Debug)]
pub enum NmiMessage {
    // vm_id
    BootVm(usize),
}

impl NmiMsgQueue {
    const fn new() -> Self {
        Self {
            msg_queue: LinkedList::new(),
        }
    }
    pub fn push(&mut self, ipi_msg: NmiMessage) {
        self.msg_queue.push_back(ipi_msg);
    }

    pub fn pop(&mut self) -> Option<NmiMessage> {
        self.msg_queue.pop_front()
    }
}

pub fn nmi_send_msg(target_cpu_id: usize, msg: NmiMessage) {
    CPU_NMI_LIST[target_cpu_id].lock().push(msg);
    // send ipi to target core
    axhal::irq::send_nmi_to(target_cpu_id);
}
