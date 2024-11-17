use std::sync::atomic::AtomicUsize;

use super::{ConcurrentSubQueue, CountableConcurrentSubQueue};

pub struct CountableWrapper<S> {
    queue: S,
    enq_count: AtomicUsize,
    deq_count: AtomicUsize,
}

impl<S, T> ConcurrentSubQueue<T> for CountableWrapper<S>
where
    S: ConcurrentSubQueue<T>,
{
    type LockType = S::LockType;

    fn new() -> Self {
        Self {
            queue: S::new(),
            enq_count: 0.into(),
            deq_count: 0.into(),
        }
    }

    fn new_lock() -> Self::LockType {
        S::new_lock()
    }

    fn enqueue(&self, item: T, lock_type: &mut Self::LockType) {
        self.enq_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.queue.enqueue(item, lock_type)
    }

    fn dequeue(&self, lock_type: &mut Self::LockType) -> Option<T> {
        if let Some(item) = self.queue.dequeue(lock_type) {
            self.deq_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Some(item)
        } else {
            None
        }
    }
}

impl<S, T> CountableConcurrentSubQueue<T> for CountableWrapper<S>
where
    S: ConcurrentSubQueue<T>,
{
    fn enq_count(&self) -> usize {
        self.enq_count.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn deq_count(&self) -> usize {
        self.deq_count.load(std::sync::atomic::Ordering::Relaxed)
    }
}
