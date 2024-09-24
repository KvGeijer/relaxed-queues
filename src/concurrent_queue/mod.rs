pub mod ms;

pub trait ConcurrentQueue<T> {
    /// Creates a new concurrent queue, with default configuration
    fn new() -> Self;

    /// Returns a thread handle to the queue, which can be used for enqueues and dequeues
    fn register(&self) -> impl Handle<T>;
}

pub trait Handle<T> {
    fn enqueue(&mut self, item: T);

    fn dequeue(&mut self) -> Option<T>;
}
