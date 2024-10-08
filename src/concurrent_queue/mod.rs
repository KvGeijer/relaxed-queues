pub mod ms;

pub trait ConcurrentSubQueue<T> {
    type LockType;
    /// Creates a new concurrent queue, with default configuration
    fn new() -> Self;
    fn new_lock() -> Self::LockType;
    fn enqueue(&self, item: T, lock_type: &mut Self::LockType);
    fn dequeue(&self, lock_type: &mut Self::LockType) -> Option<T>;
}
