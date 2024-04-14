use spin::Mutex;

extern crate alloc;
use alloc::vec::Vec;

const MAX_CPUS: usize = 8;

pub static CPU_NMI_LIST: Mutex<Vec<NmiMsgQueue>> = Mutex::new(Vec::new());
 

pub struct NmiMsgQueue {
    pub msg_queue: Vec<NmiMessage>,
}


#[derive(Copy, Clone, Debug)]
pub enum NmiMessage {
    NIMBOS(usize, usize),
}


impl NmiMsgQueue {
    pub fn default() -> NmiMsgQueue {
        NmiMsgQueue { msg_queue: Vec::new() }
    }

    pub fn push(&mut self, ipi_msg: NmiMessage) {
        self.msg_queue.push(ipi_msg);
    }

    pub fn pop(&mut self) -> Option<NmiMessage> {
        self.msg_queue.pop()
    }
}

pub fn nmi_send_msg(target_id: usize, msg: NmiMessage) {
    
    let mut cpu_nmi_list = CPU_NMI_LIST.lock();
    if cpu_nmi_list.len() < target_id+1 {
        for _ in cpu_nmi_list.len()..target_id+1 { // need to get cpu num by config
            cpu_nmi_list.push(NmiMsgQueue::default());
        }
    }
    cpu_nmi_list[target_id].msg_queue.push(msg);
    debug!("cpu_int_list {:?}", cpu_nmi_list[target_id].msg_queue);
    // send ipi to target core
    axhal::irq::send_nmi_to(target_id);
}
