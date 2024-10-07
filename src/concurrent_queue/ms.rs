use std::{mem::ManuallyDrop, ops::Deref};

use haphazard::{raw::Pointer, AtomicPtr, HazardPointer};

use super::{ConcurrentQueue, Handle};

struct Node<T> {
    next: AtomicPtr<Node<T>>,
    data: ManuallyDrop<T>,
}

impl<T> Node<T> {
    fn new(data: T) -> Self {
        Self {
            next: unsafe { AtomicPtr::new(core::ptr::null_mut()) },
            data: ManuallyDrop::new(data),
        }
    }
}

pub struct MSQueue<T> {
    head: AtomicPtr<Node<T>>,
    tail: AtomicPtr<Node<T>>,
}

impl<T> MSQueue<T> {
    pub fn new() -> Self {
        let sentinel = Box::new(Node::new(unsafe { std::mem::zeroed() })).into_raw();
        // TODO drop all data in queue when queue dropped
        Self {
            head: unsafe { AtomicPtr::new(sentinel) },
            tail: unsafe { AtomicPtr::new(sentinel) },
        }
    }
}

impl<T: Sync + Send> MSQueue<T> {
    pub fn enqueue(&self, hp: &mut HazardPointer, data: T) {
        let new_node: *mut Node<T> = Box::new(Node::new(data)).into_raw();

        let mut tail;
        loop {
            tail = self.tail.safe_load(hp).unwrap();
            // Remove if? We think it is an optimization.
            if std::ptr::eq(tail, self.tail.load_ptr()) {
                if std::ptr::eq(tail.next.load_ptr(), std::ptr::null_mut()) {
                    // Why did it not work with compare_exchange here?
                    if unsafe {
                        tail.next
                            .compare_exchange_ptr(std::ptr::null_mut(), new_node)
                    }
                    .is_ok()
                    {
                        break;
                    }
                } else {
                    unsafe {
                        let _ = self.tail.compare_exchange_ptr(
                            tail as *const Node<T> as *mut Node<T>,
                            tail.next.load_ptr(),
                        );
                    };
                }
            };
        }
        unsafe {
            let _ = self
                .tail
                .compare_exchange_ptr(tail as *const Node<T> as *mut Node<T>, new_node);
        }
    }

    pub fn dequeue(&self, hp_head: &mut HazardPointer, hp_next: &mut HazardPointer) -> Option<T> {
        let mut next_ptr;
        loop {
            let head = self
                .head
                .safe_load(hp_head)
                .expect("MS queue should never be empty");
            let head_ptr = head as *const Node<T>;
            let tail_ptr = self.tail.load_ptr();

            if head_ptr == self.head.load_ptr() {
                next_ptr = head.next.load_ptr();
                if head_ptr == tail_ptr {
                    if next_ptr.is_null() {
                        // Empty
                        return None;
                    } else {
                        // Help the partially completed enqueue
                        unsafe {
                            let _ = self
                                .tail
                                .compare_exchange_ptr(tail_ptr as *mut Node<T>, next_ptr);
                        }
                    }
                } else {
                    // Non-empty, read next value
                    let next = head.next.safe_load(hp_next).unwrap();
                    match unsafe {
                        self.head
                            .compare_exchange_ptr(head_ptr as *mut Node<T>, next_ptr)
                    } {
                        Ok(unlinked_head_ptr) => {
                            unsafe {
                                unlinked_head_ptr.unwrap().retire();
                            }

                            // Take and return ownership of the data.
                            // Algorithm guarantees we never read this data again.
                            return Some(unsafe { std::ptr::read(next.data.deref() as *const _) });
                        }
                        Err(_new_next) => {}
                    }
                }
            }
        }
    }
}

impl<T: Send + Sync> ConcurrentQueue<T> for MSQueue<T> {
    fn new() -> Self {
        MSQueue::new()
    }

    fn register(&self) -> impl Handle<T> {
        QueueHandle::new(self)
    }
}

pub struct QueueHandle<'q, T> {
    hz1: HazardPointer<'static>,
    hz2: HazardPointer<'static>,
    queue: &'q MSQueue<T>,
}

impl<'q, T: Sync + Send> QueueHandle<'q, T> {
    pub fn new(queue: &'q MSQueue<T>) -> Self {
        Self {
            hz1: HazardPointer::new(),
            hz2: HazardPointer::new(),
            queue,
        }
    }

    pub fn enqueue(&mut self, data: T) {
        self.queue.enqueue(&mut self.hz1, data);
    }

    pub fn dequeue(&mut self) -> Option<T> {
        self.queue.dequeue(&mut self.hz1, &mut self.hz2)
    }
}

impl<T: Send + Sync> Handle<T> for QueueHandle<'_, T> {
    fn enqueue(&mut self, item: T) {
        QueueHandle::enqueue(self, item);
    }

    fn dequeue(&mut self) -> Option<T> {
        QueueHandle::dequeue(self)
    }
}

#[cfg(test)]
mod test {
    use std::sync::Mutex;

    use super::{MSQueue, QueueHandle};

    #[test]
    fn simple_test() {
        let queue = MSQueue::new();
        let mut qh = QueueHandle::new(&queue);
        qh.enqueue(5);
        assert_eq!(qh.dequeue(), Some(5));
        assert_eq!(qh.dequeue(), None);
    }

    #[test]
    fn many_elem_test() {
        let queue = MSQueue::new();
        let mut qh = QueueHandle::new(&queue);
        for i in 0..5 {
            qh.enqueue(i);
        }
        assert_eq!(qh.dequeue(), Some(0));
        assert_eq!(qh.dequeue(), Some(1));
        for i in 5..10 {
            qh.enqueue(i);
        }
        for i in 2..10 {
            assert_eq!(qh.dequeue(), Some(i));
        }
        assert_eq!(qh.dequeue(), None);
        assert_eq!(qh.dequeue(), None);
        assert_eq!(qh.dequeue(), None);
        assert_eq!(qh.dequeue(), None);
    }

    #[test]
    fn simple_multi_threaded_enqueue_test() {
        let queue = MSQueue::new();
        std::thread::scope(|s| {
            let queue = &queue;
            for c in 0..3 {
                s.spawn(move || {
                    let mut qh = QueueHandle::new(queue);
                    for i in (c * 100)..((c + 1) * 100) {
                        qh.enqueue(i);
                    }
                });
            }
        });

        let mut qh = QueueHandle::new(&queue);
        let mut next_expected = [0, 100, 200];
        for _ in 0..300 {
            let val = qh.dequeue().expect("should have more elements");
            assert_eq!(next_expected[val / 100], val);
            next_expected[val / 100] = val + 1;
        }
        assert_eq!(qh.dequeue(), None);
    }

    #[test]
    fn multi_threaded_check_all_exists() {
        let queue = MSQueue::new();
        std::thread::scope(|s| {
            let queue = &queue;
            for c in 0..10 {
                s.spawn(move || {
                    let mut qh = QueueHandle::new(queue);
                    for i in (c * 100)..((c + 1) * 100) {
                        qh.enqueue(i);
                    }
                });
            }
            for _ in 0..10 {
                s.spawn(move || {
                    let mut qh = QueueHandle::new(queue);
                    let mut successful = 0;
                    while successful < 100 {
                        let val = qh.dequeue();
                        if let Some(val) = val {
                            successful += 1;
                            qh.enqueue(val);
                        }
                    }
                });
            }
        });
        let collected_elements = Mutex::new(Vec::new());
        std::thread::scope(|s| {
            for _ in 0..10 {
                s.spawn(|| {
                    let mut qh = QueueHandle::new(&queue);
                    for _ in 0..100 {
                        let val = qh.dequeue();
                        if let Some(val) = val {
                            qh.enqueue(val);
                        }
                    }
                });
            }
            for _ in 0..10 {
                s.spawn(|| {
                    let mut qh = QueueHandle::new(&queue);
                    while let Some(v) = qh.dequeue() {
                        collected_elements.lock().unwrap().push(v);
                    }
                });
            }
        });
        let mut qh = QueueHandle::new(&queue);
        let mut collected_elements = collected_elements.lock().unwrap();
        while let Some(v) = qh.dequeue() {
            collected_elements.push(v);
        }
        assert_eq!(collected_elements.len(), 1000);
        collected_elements.sort_unstable();
        for i in 0..1000 {
            assert_eq!(collected_elements[i], i);
        }
    }
}
