pub mod relaxed_queues;
pub mod strict_queues;

pub trait QueueType {}

pub struct Relaxed;
impl QueueType for Relaxed {}
pub struct Strict;
impl QueueType for Strict {}

pub trait ConcurrentQueue<T> {
    type QueueType: QueueType;
    /// Returns a thread handle to the queue, which can be used for enqueues and dequeues
    fn register(&self) -> impl Handle<T>;
}

pub trait Handle<T> {
    fn enqueue(&mut self, item: T);

    fn dequeue(&mut self) -> Option<T>;
}
