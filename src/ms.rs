use haphazard::{raw::Pointer, AtomicPtr, HazardPointer};

struct Node<T> {
    next: AtomicPtr<Node<T>>,
    data: T,
}

impl<T> Node<T> {
    fn new(data: T) -> Self {
        Self {
            next: unsafe { AtomicPtr::new(core::ptr::null_mut()) },
            data,
        }
    }
}

pub struct MSQueue<T> {
    head: AtomicPtr<Node<T>>,
    tail: AtomicPtr<Node<T>>,
}

// pub struct MSQueueHandle<T, 'ms> {
//     queue: &'ms Ms
// }

impl<T: Default + Sync> MSQueue<T> {
    pub fn new() -> Self {
        let sentinel = Box::new(Node::new(T::default())).into_raw();
        Self {
            head: unsafe { AtomicPtr::new(sentinel) },
            tail: unsafe { AtomicPtr::new(sentinel) },
        }
    }

    pub fn enqueue(&self, hp: &mut HazardPointer, data: T) {
        let mut new_node = Box::new(Node::new(data));
        let new_node_ptr = new_node.as_mut() as *mut Node<T>;

        let mut tail;
        loop {
            tail = self.tail.safe_load(hp).unwrap();
            if std::ptr::eq(tail, self.tail.load_ptr()) {
                if std::ptr::eq(tail.next.load_ptr(), std::ptr::null_mut()) {
                    // Try swapping this compare_exchange with compare_exchange_ptr
                    // at the bottom of this function?
                    match tail.next.compare_exchange(std::ptr::null_mut(), new_node) {
                        Ok(_) => break,
                        Err(v) => new_node = v,
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
                .compare_exchange_ptr(tail as *const Node<T> as *mut Node<T>, new_node_ptr);
        }
    }
}
