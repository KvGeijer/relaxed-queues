use std::{marker::PhantomData, mem::MaybeUninit};

use rand::{rngs::ThreadRng, Rng};

use crate::{
    strict_queues::{ConcurrentSubQueue, CountableVersionedConcurrentSubQueue},
    ConcurrentQueue, Handle, Relaxed,
};

pub struct DCBOQueue<SubQueue, T> {
    subqueues: Vec<SubQueue>,
    d: usize,
    // TODO try the other solution variant:
    // https://users.rust-lang.org/t/dealing-with-unconstrained-type-parameters-in-impl-blocks/49138/3
    _phantom_data: PhantomData<T>,
}

impl<T, S: CountableVersionedConcurrentSubQueue<T>> DCBOQueue<S, T> {
    fn enqueue(&self, lock: &mut S::LockType, rng: &mut ThreadRng, item: T) {
        let queue = (0..self.d)
            .map(|_| rng.gen_range(0..self.subqueues.len()))
            .map(|i| &self.subqueues[i])
            .min_by_key(|q| q.enq_count())
            .expect("should contain at least one queue");
        queue.enqueue(item, lock);
    }

    fn dequeue(&self, lock: &mut S::LockType, rng: &mut ThreadRng) -> Option<T> {
        let queue_index = (0..self.d)
            .map(|_| rng.gen_range(0..self.subqueues.len()))
            .max_by_key(|&i| self.subqueues[i].deq_count())
            .expect("should contain at least one queue");
        let item = self.subqueues[queue_index].dequeue(lock);
        if item.is_some() {
            item
        } else {
            self.double_collect(queue_index, lock)
        }
    }
}

struct DCBOQueueHandle<'queue, S: CountableVersionedConcurrentSubQueue<T>, T> {
    queue: &'queue DCBOQueue<S, T>,
    lock: S::LockType,
    thread_rng: ThreadRng,
}

impl<S: CountableVersionedConcurrentSubQueue<T>, T> Handle<T> for DCBOQueueHandle<'_, S, T> {
    fn enqueue(&mut self, item: T) {
        self.queue
            .enqueue(&mut self.lock, &mut self.thread_rng, item);
    }

    fn dequeue(&mut self) -> Option<T> {
        self.queue.dequeue(&mut self.lock, &mut self.thread_rng)
    }
}

impl<T, S: CountableVersionedConcurrentSubQueue<T>> ConcurrentQueue<T> for DCBOQueue<S, T> {
    type QueueType = Relaxed;

    fn register(&self) -> impl Handle<T> {
        DCBOQueueHandle {
            queue: self,
            lock: S::new_lock(),
            // TODO make deterministic seeding possible? (is this even useful)
            thread_rng: rand::thread_rng(),
        }
    }
}

impl<T, S: CountableVersionedConcurrentSubQueue<T>> DCBOQueue<S, T> {
    pub fn new(queue_count: usize, d: usize) -> Self {
        Self {
            subqueues: (0..queue_count).map(|_| S::new()).collect(),
            d,
            _phantom_data: PhantomData,
        }
    }

    fn double_collect(&self, index: usize, lock: &mut S::LockType) -> Option<T> {
        // fallback to checking all queues
        let mut versions = vec![MaybeUninit::uninit(); self.subqueues.len()];
        let mut start_index = index;
        'outer: loop {
            for queue_index in (start_index..).chain(0..start_index) {
                let queue = &self.subqueues[queue_index];
                versions[queue_index].write(queue.enq_version());
                if let Some(item) = queue.dequeue(lock) {
                    return Some(item);
                }
            }

            for queue_index in (start_index..).chain(0..start_index) {
                let queue = &self.subqueues[queue_index];
                if unsafe { versions[queue_index].assume_init() } != queue.enq_version() {
                    start_index = queue_index;
                    continue 'outer;
                };
            }
            return None;
        }
    }
}
