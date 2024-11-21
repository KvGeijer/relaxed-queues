use std::marker::PhantomData;

use rand::{rngs::ThreadRng, Rng};

use crate::{
    strict_queues::{ConcurrentSubQueue, CountableConcurrentSubQueue},
    ConcurrentQueue, Handle, Relaxed,
};

pub struct DRaQueue<SubQueue, T> {
    subqueues: Vec<SubQueue>,
    d: usize,
    // TODO try the other solution variant:
    // https://users.rust-lang.org/t/dealing-with-unconstrained-type-parameters-in-impl-blocks/49138/3
    _phantom_data: PhantomData<T>,
}

impl<T, S: CountableConcurrentSubQueue<T>> DRaQueue<S, T> {
    fn enqueue(&self, lock: &mut S::LockType, rng: &mut ThreadRng, item: T) {
        let queue = (0..self.d)
            .map(|_| rng.gen_range(0..self.subqueues.len()))
            .map(|i| &self.subqueues[i])
            .min_by_key(|q| q.enq_count().saturating_sub(q.deq_count()))
            .expect("should contain at least one queue");
        queue.enqueue(item, lock);
    }

    fn dequeue(&self, lock: &mut S::LockType, rng: &mut ThreadRng) -> Option<T> {
        let queue = (0..self.d)
            .map(|_| rng.gen_range(0..self.subqueues.len()))
            .map(|i| &self.subqueues[i])
            .max_by_key(|q| q.enq_count().saturating_sub(q.deq_count()))
            .expect("should contain at least one queue");
        queue.dequeue(lock)
    }
}

struct DraQueueHandle<'queue, S: CountableConcurrentSubQueue<T>, T> {
    queue: &'queue DRaQueue<S, T>,
    lock: S::LockType,
    thread_rng: ThreadRng,
}

impl<S: CountableConcurrentSubQueue<T>, T> Handle<T> for DraQueueHandle<'_, S, T> {
    fn enqueue(&mut self, item: T) {
        self.queue
            .enqueue(&mut self.lock, &mut self.thread_rng, item);
    }

    fn dequeue(&mut self) -> Option<T> {
        self.queue.dequeue(&mut self.lock, &mut self.thread_rng)
    }
}

impl<T, S: CountableConcurrentSubQueue<T>> ConcurrentQueue<T> for DRaQueue<S, T> {
    type QueueType = Relaxed;

    fn register(&self) -> impl Handle<T> {
        DraQueueHandle {
            queue: self,
            lock: S::new_lock(),
            // TODO make deterministic seeding possible? (is this even useful)
            thread_rng: rand::thread_rng(),
        }
    }
}

impl<T, S: ConcurrentSubQueue<T>> DRaQueue<S, T> {
    pub fn new(queue_count: usize, d: usize) -> Self {
        Self {
            subqueues: (0..queue_count).map(|_| S::new()).collect(),
            d,
            _phantom_data: PhantomData,
        }
    }
}
