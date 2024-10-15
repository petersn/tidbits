use std::sync::Mutex;
use std::sync::atomic::{AtomicPtr, Ordering};

const RLX: Ordering = Ordering::Relaxed;

struct Node {
    data: i32,
    next: AtomicPtr<Node>,
}

struct StupidQueueKishkes {
    tail: *mut Node,
    head: *mut Node,
}

struct StupidQueue(Mutex<StupidQueueKishkes>);

impl StupidQueue {
    fn new() -> Self {
        Self(Mutex::new(StupidQueueKishkes {
            tail: std::ptr::null_mut(),
            head: std::ptr::null_mut(),
        }))
    }

    fn push(&self, node: *mut Node) {
        let mut guard = self.0.lock().unwrap();
        if !guard.tail.is_null() {
            let tail = unsafe { &mut *guard.tail };
            tail.next.store(node, RLX);
        }
        guard.tail = node;
        if guard.head.is_null() {
            guard.head = node;
        }
    }

    fn pop(&self) -> Option<*mut Node> {
        let mut guard = self.0.lock().unwrap();
        if guard.head.is_null() {
            return None;
        }
        let popped_ptr = guard.head;
        let head = unsafe { &mut *popped_ptr };
        guard.head = head.next.load(RLX);
        Some(head)
    }
}

struct Queue {
    tail: AtomicPtr<Node>,
    head: AtomicPtr<Node>,
}

impl Queue {
    fn new() -> Self {
        Self {
            tail: AtomicPtr::new(std::ptr::null_mut()),
            head: AtomicPtr::new(std::ptr::null_mut()),
        }
    }

    fn push(&self, node: *mut Node) {
        let tail = self.tail.load(RLX);
        if !tail.is_null() {
            let tail_node = unsafe { &mut *tail };
            tail_node.next.store(node, RLX);
        }
        self.tail.store(node, RLX);
        if self.head.load(RLX).is_null() {
            self.head.store(node, RLX);
        }
    }

    fn pop(&self) -> Option<*mut Node> {
        let head = self.head.load(RLX);
        if head.is_null() {
            return None;
        }
        let head_node = unsafe { &mut *head };
        self.head.store(head_node.next.load(RLX), RLX);
        Some(head)
    }
}

fn make_node(data: i32) -> *mut Node {
    Box::into_raw(Box::new(Node {
        data,
        next: AtomicPtr::new(std::ptr::null_mut()),
    }))
}

fn get_val(x: *mut Node) -> i32 {
    unsafe { (*x).data }
}

fn main() {
    let queue = Queue::new();
    for i in 1..=5 {
        queue.push(make_node(i));
    }
    while let Some(node) = queue.pop() {
        println!("Got: {}", get_val(node));
    }
}
