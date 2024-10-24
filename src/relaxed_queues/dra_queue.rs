use std::{
    marker::PhantomData,
    sync::atomic::{AtomicUsize, Ordering},
};

use rand::{rngs::ThreadRng, Rng};

use crate::{concurrent_queue::ConcurrentSubQueue, ConcurrentQueue, Handle, Relaxed};

pub struct DRaQueue<SubQueue, T> {
    subqueues: Vec<(SubQueue, AtomicUsize)>,
    d: usize,
    // TODO try the other solution variant:
    // https://users.rust-lang.org/t/dealing-with-unconstrained-type-parameters-in-impl-blocks/49138/3
    _phantom_data: PhantomData<T>,
}

impl<T, S: ConcurrentSubQueue<T>> DRaQueue<S, T> {
    fn enqueue(&self, lock: &mut S::LockType, rng: &mut ThreadRng, item: T) {
        let (queue, queue_len) = (0..self.d)
            .map(|_| rng.gen_range(0..self.subqueues.len()))
            .map(|i| &self.subqueues[i])
            .min_by_key(|(_, a)| a.load(Ordering::Relaxed))
            .expect("should contain at least one queue");
        queue_len.fetch_add(1, Ordering::SeqCst);
        queue.enqueue(item, lock);
    }

    fn dequeue(&self, lock: &mut S::LockType, rng: &mut ThreadRng) -> Option<T> {
        let (queue, queue_len) = (0..self.d)
            .map(|_| rng.gen_range(0..self.subqueues.len()))
            .map(|i| &self.subqueues[i])
            .max_by_key(|(_, a)| a.load(Ordering::Relaxed))
            .expect("should contain at least one queue");
        match queue.dequeue(lock) {
            Some(val) => {
                queue_len.fetch_sub(1, Ordering::SeqCst);
                Some(val)
            }
            None => None,
        }
    }
}

struct DraQueueHandle<'queue, S: ConcurrentSubQueue<T>, T> {
    queue: &'queue DRaQueue<S, T>,
    lock: S::LockType,
    thread_rng: ThreadRng,
}

impl<S: ConcurrentSubQueue<T>, T> Handle<T> for DraQueueHandle<'_, S, T> {
    fn enqueue(&mut self, item: T) {
        self.queue
            .enqueue(&mut self.lock, &mut self.thread_rng, item);
    }

    fn dequeue(&mut self) -> Option<T> {
        self.queue.dequeue(&mut self.lock, &mut self.thread_rng)
    }
}

impl<T, S: ConcurrentSubQueue<T>> ConcurrentQueue<T> for DRaQueue<S, T> {
    type QueueType = Relaxed;

    fn new() -> Self {
        const INITIAL_QUEUE_COUNT: usize = 8;
        const INITIAL_D: usize = 2;
        Self::new_with_config(INITIAL_QUEUE_COUNT, INITIAL_D)
    }

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
    pub fn new_with_config(queue_count: usize, d: usize) -> Self {
        Self {
            subqueues: (0..queue_count)
                .map(|_| (S::new(), AtomicUsize::new(0)))
                .collect(),
            d,
            _phantom_data: PhantomData,
        }
    }
}
