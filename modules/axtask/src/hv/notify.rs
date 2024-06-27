//! notify cores
use alloc::{collections::VecDeque, sync::Arc, vec, vec::Vec};
use alloc::collections::LinkedList;
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::{Mutex, Once};
use x86::current::vmx::vmclear;
use x86::segmentation::ds;
use crate::hv::vm::config::BSP_CPU_ID;

pub const HV_MSG: usize = 233;

/// global
static HV_MSG_LISTS: Once<Mutex<MsgLists>> = Once::new();

/// init
pub fn init_mgs_lists(cap: usize) {
    HV_MSG_LISTS.call_once(|| Mutex::new(MsgLists::new(cap)));
}

/// fn get
pub fn receive_message(hart_id: usize) -> Option<Message> {
    HV_MSG_LISTS
        .get()
        .unwrap()
        .lock()
        .receive_message(hart_id)
}

/// send
pub fn send_message(msg: Message) {
    HV_MSG_LISTS.get().unwrap().lock().send_message(msg);
}

/// broadcast
pub fn broadcast_message(msg: Message) {
    HV_MSG_LISTS.get().unwrap().lock().broadcast_message(msg)
}

pub fn wait_on_reply(msg: &Message) -> bool {
    !HV_MSG_LISTS.get().unwrap().lock().wait_reply(&msg)
}


/// msgs
#[derive(Debug)]
pub struct MsgLists {
    messages: Vec<LinkedList<Message>>,
}

impl MsgLists {
    /// new
    pub fn new(cap: usize) -> Self {
        let mut vec = Vec::with_capacity(cap);
        for _ in 0..cap {
            vec.push(LinkedList::new());
        }
        Self {
            messages: vec,
        }
    }
    /// get_message
    pub fn receive_message(&mut self, hart_id: usize) -> Option<Message> {
        self.messages[hart_id].pop_front()
    }
    /// push
    pub fn send_message(&mut self, msg: Message) {
        self.messages[msg.dest].push_back(msg);
    }
    /// message's dest should be bsp
    pub fn broadcast_message(&mut self, msg: Message) {
        for (i, que) in self.messages.iter_mut().enumerate() {
            if i == BSP_CPU_ID {
                continue;
            }
            let mut msg = msg.clone();
            msg.dest = i;
            que.push_back(msg.clone());
        }
    }

    /// assume only one
    pub fn wait_reply(&mut self, expected: &Message) -> bool {
        let dst = expected.dest;
        let x = self.messages[dst].extract_if(|m| m == expected).collect::<Vec<Message>>();
        !x.is_empty()
    }
}


static MSG_ID_ALLOCATOR: AtomicUsize = AtomicUsize::new(0);


/// msg
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Message {
    pub id: usize,
    /// dest physical cpu
    pub dest: usize,
    /// source physical cpu
    pub src: usize,
    /// signal
    pub signal: Signal,
    /// args
    pub args: Vec<usize>,
}

impl Message {
    /// new
    pub fn new(src: usize, dest: usize, signal: Signal, args: Vec<usize>) -> Self {
        Self { id: MSG_ID_ALLOCATOR.fetch_add(1, Ordering::Relaxed), dest, src, signal, args }
    }

    pub fn new_reply(msg: &Self) -> Self {
        Self {
            id: msg.id,
            dest: msg.src,
            src: msg.dest,
            signal: Signal::Ok,
            args: vec![],
        }
    }
}

/// sig
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Signal {
    Clear,
    Ok,
}

#[no_mangle]
pub fn hv_msg_handler(hart_id: usize) {
    error!("in hv msg handler, hart_id {}",hart_id);
    let mut guard = HV_MSG_LISTS.get().unwrap().lock();
    if let Some(msg) = guard.receive_message(hart_id) {
        match msg.signal {
            Signal::Clear => {
                let addr = msg.args[0];
                unsafe {
                    error!("{} vmclear {}", hart_id, addr);
                    vmclear(addr as u64).unwrap();
                }
                // reply
                let reply = Message::new_reply(&msg);
                guard.send_message(reply);
            }
            _ => {
                panic!("unknown msg");
            }
        }
    }
}
