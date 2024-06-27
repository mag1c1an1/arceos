use alloc::{collections::VecDeque, sync::Arc};
use core::ops::Deref;
use core::sync::atomic::{AtomicIsize, Ordering};
use scheduler::BaseScheduler;
use crate::TaskInner;


/// A task wrapper for the [`HVScheduler`].
///
/// It adds a time slice counter to use in round-robin scheduling.
pub struct HVTask<const MAX_TIME_SLICE: usize> {
    inner: TaskInner,
    time_slice: AtomicIsize,
}

impl<const S: usize> HVTask<S> {
    /// Creates a new [`HVTask`] from the inner task struct.
    pub const fn new(inner: TaskInner) -> Self {
        Self {
            inner,
            time_slice: AtomicIsize::new(S as isize),
        }
    }

    fn time_slice(&self) -> isize {
        self.time_slice.load(Ordering::Acquire)
    }

    fn reset_time_slice(&self) {
        self.time_slice.store(S as isize, Ordering::Release);
    }

    /// Returns a reference to the inner task struct.
    pub const fn inner(&self) -> &TaskInner {
        &self.inner
    }

    fn bind_on_curr_cpu(&self) -> bool {
        self.inner.bind_on_curr_cpu()
    }
}

impl<const S: usize> Deref for HVTask<S> {
    type Target = TaskInner;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// A simple [Round-Robin] (RR) preemptive scheduler.
///
/// It's very similar to the [`FifoScheduler`], but every task has a time slice
/// counter that is decremented each time a timer tick occurs. When the current
/// task's time slice counter reaches zero, the task is preempted and needs to
/// be rescheduled.
///
/// Unlike [`FifoScheduler`], it uses [`VecDeque`] as the ready queue. So it may
/// take O(n) time to remove a task from the ready queue.
///
/// [Round-Robin]: https://en.wikipedia.org/wiki/Round-robin_scheduling
/// [`FifoScheduler`]: crate::FifoScheduler
pub struct HVScheduler<const MAX_TIME_SLICE: usize> {
    ready_queue: VecDeque<Arc<HVTask<MAX_TIME_SLICE>>>,
}

impl<const S: usize> HVScheduler<S> {
    /// Creates a new empty [`HVScheduler`].
    pub const fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    /// get the name of scheduler
    pub fn scheduler_name() -> &'static str {
        "HV Specific"
    }
}

impl<const S: usize> BaseScheduler for HVScheduler<S> {
    type SchedItem = Arc<HVTask<S>>;

    fn init(&mut self) {}

    fn add_task(&mut self, task: Self::SchedItem) {
        self.ready_queue.push_back(task);
    }

    fn remove_task(&mut self, task: &Self::SchedItem) -> Option<Self::SchedItem> {
        // TODO: more efficient
        self.ready_queue
            .iter()
            .position(|t| Arc::ptr_eq(t, task))
            .and_then(|idx| self.ready_queue.remove(idx))
    }

    fn pick_next_task(&mut self) -> Option<Self::SchedItem> {
        // self.ready_queue.pop_front()
        self.ready_queue
            .iter()
            .position(|t| t.bind_on_curr_cpu())
            .and_then(|idx| self.ready_queue.remove(idx))
    }

    fn put_prev_task(&mut self, prev: Self::SchedItem, preempt: bool) {
        if prev.time_slice() > 0 && preempt {
            self.ready_queue.push_front(prev)
        } else {
            prev.reset_time_slice();
            self.ready_queue.push_back(prev)
        }
    }

    /// for normal
    fn task_tick(&mut self, current: &Self::SchedItem) -> bool {
        let old_slice = current.time_slice.fetch_sub(1, Ordering::Release);
        old_slice <= 1
    }
    // fn task_tick(&mut self, current: &Self::SchedItem) -> bool {
    //     if !current.is_vcpu_task() {
    //         let old_slice = current.time_slice.fetch_sub(1, Ordering::Release);
    //         old_slice <= 1
    //     } else {
    //         current.time_slice.load(Ordering::Acquire) <= 1
    //     }
    // }

    fn set_priority(&mut self, _task: &Self::SchedItem, _prio: isize) -> bool {
        false
    }
}

impl<const S: usize> HVScheduler<S> {
    pub fn vcpu_task_tick(&mut self, current: &Arc<HVTask<S>>) -> bool {
        if current.is_vcpu_task() {
            let old_slice = current.time_slice.fetch_sub(1, Ordering::Release);
            old_slice <= 1
        } else {
            current.time_slice.load(Ordering::Acquire) <= 1
        }
    }
}