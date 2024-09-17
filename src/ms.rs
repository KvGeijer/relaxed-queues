use haphazard::{raw::Pointer, AtomicPtr, HazardPointer};

struct Node<T> {
    next: AtomicPtr<Node<T>>,
    data: T, // TODO: Don't drop on free
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

impl<T: Default + Sync + Send> MSQueue<T> {
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
            // Remove if? We think it is an optimization.
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
                    // Non-empty
                    // Read next value
                    let next = head.next.safe_load(hp_next).unwrap();
                    match unsafe {
                        self.head
                            .compare_exchange_ptr(head_ptr as *mut Node<T>, next_ptr)
                    } {
                        Ok(unlinked_head_ptr) => {
                            unsafe {
                                // Retire head_ptr (TODO: Don't free data: T)
                                unlinked_head_ptr.unwrap().retire();
                            }

                            // Should return next.value, by value
                            return Some(std::mem::replace(
                                unsafe { (&next.data as *const T as *mut T).as_mut().unwrap() },
                                unsafe { std::mem::zeroed() },
                            ));
                        }
                        Err(_new_next) => {}
                    }
                }
            }
        }
    }
}
