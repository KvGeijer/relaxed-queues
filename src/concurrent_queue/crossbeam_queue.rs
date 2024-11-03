use crossbeam_queue::SegQueue;

use crate::{ConcurrentQueue, Strict};

use super::ConcurrentSubQueue;

pub struct Handle<'a, T> {
    queue: &'a SegQueue<T>,
}

impl<'a, T> crate::Handle<T> for Handle<'a, T> {
    fn enqueue(&mut self, item: T) {
        self.queue.push(item);
    }

    fn dequeue(&mut self) -> Option<T> {
        self.queue.pop()
    }
}

impl<T> ConcurrentQueue<T> for SegQueue<T> {
    type QueueType = Strict;

    fn register(&self) -> impl crate::Handle<T> {
        Handle { queue: &self }
    }
}

impl<T> ConcurrentSubQueue<T> for SegQueue<T> {
    type LockType = ();

    fn new() -> Self {
        SegQueue::new()
    }

    fn new_lock() -> Self::LockType {
        ()
    }

    fn enqueue(&self, item: T, _lock_type: &mut Self::LockType) {
        self.push(item)
    }

    fn dequeue(&self, _lock_type: &mut Self::LockType) -> Option<T> {
        self.pop()
    }
}
