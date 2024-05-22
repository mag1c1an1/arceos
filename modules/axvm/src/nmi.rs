use alloc::collections::LinkedList;

use spin::Mutex;

use axconfig::SMP;

const PER_CPU_NMI_MSG_QUEUE: Mutex<NmiMsgQueue> = Mutex::new(NmiMsgQueue::new());
pub static CORE_NMI_LIST: [Mutex<NmiMsgQueue>; SMP] = [PER_CPU_NMI_MSG_QUEUE; SMP];

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

pub fn nmi_send_msg_by_core_id(target_core_id: usize, msg: NmiMessage) {
    let current_cpu = axhal::current_cpu_id();
    let target_cpu_id = axhal::core_id_to_cpu_id(target_core_id);
    match target_cpu_id {
        Some(target_cpu_id) => {
            if target_cpu_id == current_cpu {
                warn!(
                    "CPU{} try send nmi to self, something is wrong",
                    current_cpu
                );
                return;
            }
            info!(
                "CPU {} send nmi ipi to CPU{} (Linux processor ID {})",
                current_cpu, target_cpu_id, target_core_id
            );
            CORE_NMI_LIST[target_core_id].lock().push(msg);
            // Send ipi to target core through local APIC.
            axhal::irq::send_nmi_to(target_cpu_id);
        }
        None => {
            warn!("Core {} not existed, just skip it", target_core_id);
        }
    }
}
