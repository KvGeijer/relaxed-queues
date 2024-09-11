use haphazard::{raw::Pointer, AtomicPtr};

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

struct MSQueue<T> {
    head: AtomicPtr<Node<T>>,
    tail: AtomicPtr<Node<T>>,
}

impl<T: Default> MSQueue<T> {
    fn new() -> Self {
        let sentinel = Box::new(Node::new(T::default())).into_raw();
        Self {
            head: unsafe { AtomicPtr::new(sentinel) },
            tail: unsafe { AtomicPtr::new(sentinel) },
        }
    }
}
