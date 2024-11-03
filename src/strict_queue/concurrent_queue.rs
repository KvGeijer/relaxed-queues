use crate::{ConcurrentQueue, Strict};

use super::ConcurrentSubQueue;

impl<T: Sync + Send> ConcurrentSubQueue<T> for concurrent_queue::ConcurrentQueue<T> {
    type LockType = ();

    fn new() -> Self {
        concurrent_queue::ConcurrentQueue::unbounded()
    }

    fn new_lock() -> Self::LockType {
        ()
    }

    fn enqueue(&self, item: T, _lock_type: &mut Self::LockType) {
        let _ = self.push(item);
    }

    fn dequeue(&self, _lock_type: &mut Self::LockType) -> Option<T> {
        self.pop().ok()
    }
}

struct Handle<'q, T> {
    queue: &'q concurrent_queue::ConcurrentQueue<T>,
}

impl<T: Sync + Send> ConcurrentQueue<T> for concurrent_queue::ConcurrentQueue<T> {
    type QueueType = Strict;

    fn register(&self) -> impl crate::Handle<T> {
        Handle { queue: &self }
    }
}

impl<T: Sync + Send> crate::Handle<T> for Handle<'_, T> {
    fn enqueue(&mut self, item: T) {
        let _ = self.queue.push(item);
    }

    fn dequeue(&mut self) -> Option<T> {
        self.queue.pop().ok()
    }
}
