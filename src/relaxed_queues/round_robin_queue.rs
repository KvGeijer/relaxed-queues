use std::marker::PhantomData;

use crate::{strict_queues::ConcurrentSubQueue, ConcurrentQueue, Handle, Relaxed};

pub struct RoundRobinQueue<SubQueue, T> {
    subqueues: Vec<SubQueue>,
    // TODO try the other solution variant:
    // https://users.rust-lang.org/t/dealing-with-unconstrained-type-parameters-in-impl-blocks/49138/3
    _phantom_data: PhantomData<T>,
}

impl<T, S: ConcurrentSubQueue<T>> RoundRobinQueue<S, T> {
    pub fn new(queue_count: usize) -> Self {
        Self {
            subqueues: (0..queue_count).map(|_| S::new()).collect(),
            _phantom_data: PhantomData,
        }
    }
}

impl<S: ConcurrentSubQueue<T>, T> ConcurrentQueue<T> for RoundRobinQueue<S, T> {
    type QueueType = Relaxed;

    fn register(&self) -> impl Handle<T> {
        let lock = S::new_lock();
        RoundRobinQueueHandle {
            cursor: 0,
            queue: &self,
            lock,
        }
    }
}

pub struct RoundRobinQueueHandle<'q, S: ConcurrentSubQueue<T>, T> {
    cursor: usize,
    queue: &'q RoundRobinQueue<S, T>,
    lock: S::LockType,
}

impl<S: ConcurrentSubQueue<T>, T> RoundRobinQueueHandle<'_, S, T> {
    fn inc_cursor(&mut self) {
        self.cursor += 1;
        if self.cursor >= self.queue.subqueues.len() {
            self.cursor = 0;
        }
    }
}

impl<S: ConcurrentSubQueue<T>, T> Handle<T> for RoundRobinQueueHandle<'_, S, T> {
    fn enqueue(&mut self, item: T) {
        self.inc_cursor();
        self.queue.subqueues[self.cursor].enqueue(item, &mut self.lock);
    }

    fn dequeue(&mut self) -> Option<T> {
        self.inc_cursor();
        if let Some(item) = self.queue.subqueues[self.cursor].dequeue(&mut self.lock) {
            return Some(item);
        }
        // fallback to checking all queues
        for queue in self.queue.subqueues[self.cursor..]
            .iter()
            .chain(self.queue.subqueues[0..self.cursor].iter())
        {
            if let Some(item) = queue.dequeue(&mut self.lock) {
                return Some(item);
            }
        }
        None
    }
}
