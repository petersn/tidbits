use std::i32;
use std::marker::PhantomData;
use std::sync::Mutex;
use std::sync::atomic::{AtomicPtr, Ordering};

const IDK: Ordering = Ordering::Relaxed;
const SENTINEL_NONSENSE: *mut Node = 0xdeadbeef as *mut Node;

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
            tail.next.store(node, IDK);
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
        if guard.head == guard.tail {
            guard.tail = std::ptr::null_mut();
        }
        guard.head = head.next.load(IDK);
        Some(head)
    }

    fn format(&self) -> String {
        let guard = self.0.lock().unwrap();
        format!("head={:?} tail={:?}", guard.head, guard.tail)
    }
}

struct Queue {
    head: AtomicPtr<Node>,
    tail: AtomicPtr<Node>,
}

impl Queue {
    fn new() -> Self {
        Self {
            head: AtomicPtr::new(std::ptr::null_mut()),
            tail: AtomicPtr::new(std::ptr::null_mut()),
        }
    }

    fn push(&self, new_node: *mut Node) {
        // let tail = loop {
        //     let tail = self.tail.load(IDK);
        //     if self.tail.compare_exchange(
        //         tail, new_node, IDK, IDK,
        //     ).is_ok() {
        //         break tail;
        //     }
        // };
        let tail = self.tail.swap(new_node, IDK);
        if tail.is_null() {
            self.head.store(new_node, IDK);
        } else {
            let tail_node = unsafe { &mut *tail };
            if tail_node.next.compare_exchange(std::ptr::null_mut(), new_node, IDK, IDK).is_err() {
                self.head.store(new_node, IDK);
            }
        }
    }

    fn pop(&self) -> Option<*mut Node> {
        let head = self.head.load(IDK);
        if head.is_null() {
            return None;
        }
        let head_node = unsafe { &mut *head };
        let head_node_next = head_node.next.load(IDK);
        if head_node_next == SENTINEL_NONSENSE {
            return None;
        }
        if head_node_next.is_null() {
            // match head_node.next.compare_exchange(
            //     std::ptr::null_mut(), SENTINEL_NONSENSE, IDK, IDK,
            // ) {
            //     Ok(_) => {}
            //     Err(actual_next) => self.head.store(actual_next, IDK),
            // }
            let actual_next = head_node.next.swap(SENTINEL_NONSENSE, IDK);
            if actual_next != std::ptr::null_mut() {
                self.head.store(actual_next, IDK);
            }
        } else {
            self.head.store(head_node_next, IDK);
        }    
        Some(head)
    }

    fn format(&self) -> String {
        format!("head={:?} tail={:?}", self.head.load(IDK), self.tail.load(IDK))
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
    for round in 0..2 {
        for i in 1..=5 {
            queue.push(make_node(10*round + i));
        }
        while let Some(node) = queue.pop() {
            println!("Got: {}", get_val(node));
            println!("Printy: {}", queue.format());
        }
    }
}
