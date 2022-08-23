//! Implementation of [`TaskManager`]
//!
//! It is only used to manage processes and schedule process based on ready queue.
//! Other CPU process monitoring functions are in Processor.


use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
use alloc::collections::BinaryHeap;
use core::cmp::Ordering;

pub struct TaskManager {
    ready_queue: BinaryHeap<TaskControlBlockPointer>,
}

struct TaskControlBlockPointer(pub Arc<TaskControlBlock>);

impl PartialOrd for TaskControlBlockPointer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.0.inner_exclusive_access().pass.partial_cmp(&self.0.inner_exclusive_access().pass)
    }
}

impl PartialEq<Self> for TaskControlBlockPointer {
    fn eq(&self, other: &Self) -> bool {
        self.0.inner_exclusive_access().pass == other.0.inner_exclusive_access().pass
    }
}

impl Eq for TaskControlBlockPointer {

}

impl Ord for TaskControlBlockPointer {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

// YOUR JOB: FIFO->Stride
/// A simple FIFO scheduler.
impl TaskManager {
    pub fn new() -> Self {
        Self {
            ready_queue: BinaryHeap::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push(TaskControlBlockPointer(task));
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        match self.ready_queue.pop() {
            Some(taskPointer) => {
                Some(taskPointer.0)
            }
            None => None,
        }
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.exclusive_access().add(task);
}

pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.exclusive_access().fetch()
}
