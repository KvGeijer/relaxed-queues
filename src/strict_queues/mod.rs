pub mod concurrent_queue;
pub mod countable_wrapper;
pub mod crossbeam_queue;
pub mod lockfree_queue;
pub mod ms;

pub trait ConcurrentSubQueue<T> {
    type LockType;
    /// Creates a new concurrent queue, with default configuration
    fn new() -> Self;
    fn new_lock() -> Self::LockType;
    fn enqueue(&self, item: T, lock_type: &mut Self::LockType);
    fn dequeue(&self, lock_type: &mut Self::LockType) -> Option<T>;
}

pub trait CountableConcurrentSubQueue<T>: ConcurrentSubQueue<T> {
    fn enq_count(&self) -> usize;
    fn deq_count(&self) -> usize;
}

pub trait CountableVersionedConcurrentSubQueue<T>: CountableConcurrentSubQueue<T> {
    fn enq_version(&self) -> usize;
}
