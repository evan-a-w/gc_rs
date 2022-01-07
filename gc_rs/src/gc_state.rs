use std::ptr::NonNull;
use std::cell::{Cell, RefCell};
use std::time::{Duration, Instant};

use crate::traits::*;

pub struct GcState {
    list_head: Option<NonNull<GcNode<dyn Trace>>>,
    last_gc: Instant,
    pub gc_duration: Duration,
}

#[derive(Debug)]
pub struct GcNode<T: Trace + ?Sized + 'static> {
    pub data: Cell<GcData>,
    pub next: Option<NonNull<GcNode<dyn Trace>>>,
    pub val: T,
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct GcData {
    data: usize,
}

impl GcData {
    pub fn new() -> Self {
        // Start off rooted
        GcData { data: 1 }
    }

    pub fn is_root(&self) -> bool {
        self.get_roots() > 0
    }

    pub fn get_roots(&self) -> usize {
        self.data & !(1 << 63)
    }

    pub fn add_roots(&mut self) {
        // Might add checking that it's less than (1 << 63) - 1
        self.data += 1;
    }

    pub fn sub_roots(&mut self) {
        // Might add checking that it's less than (1 << 63) - 1
        if self.data &!(1 << 63) > 0 {
            self.data -= 1;
        }
    }

    pub fn mark(&mut self) {
        self.data |= 1 << 63;
    }

    pub fn unmark(&mut self) {
        self.data &= !(1 << 63);
    }

    pub fn is_marked(&self) -> bool {
        self.data & (1 << 63) != 0
    }
}

impl GcState {
    pub fn new() -> Self {
        GcState {
            list_head: None,
            last_gc: Instant::now(),
            gc_duration: Duration::from_secs(2),
        }
    }

    pub unsafe fn get_ptrs_len(&self) -> usize {
        let mut len = 0;
        let mut curr = self.list_head;
        while let Some(node) = curr {
            len += 1;
            curr = (*node.as_ptr()).next;
        }
        len
    }

    pub unsafe fn get_roots_len(&self) -> usize {
        let mut len = 0;
        let mut curr = self.list_head;
        while let Some(node) = curr {
            let r = node.as_ref();
            if r.data.get().is_root() {
                len += 1;
            }
            curr = r.next;
        }
        len
    }

    pub fn try_collect_garbage(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_gc) > self.gc_duration {
            self.last_gc = now;
            self.collect_garbage();
        }
    }

    pub fn collect_garbage(&mut self) {
        // Traverse the list and trace all nodes that have roots
        unsafe {
            let mut curr = self.list_head;
            while let Some(mut node) = curr {
                let node = node.as_mut();
                let mut data = node.data.get();
                if data.is_root() {
                    data.mark();
                    node.data.set(data); 
                    node.val.trace();
                }
                curr = node.next;
            }
        }

        // Traverse again, removing and freeing nodes that are not marked.
        // Also unmark all nodes that are marked.
        unsafe {
            let mut curr = self.list_head;
            let mut prev: Option<NonNull<GcNode<dyn Trace>>> = None;
            while let Some(mut cnode) = curr {
                let node = cnode.as_mut();
                if node.data.get().is_marked() {
                    node.data.get_mut().unmark();
                    curr = node.next;
                    prev = Some(cnode);
                } else {
                    if let Some(mut prev) = prev {
                        prev.as_mut().next = node.next;
                    } else {
                        self.list_head = node.next;
                    }
                    curr = node.next;
                    // Might free?
                    let _ = *Box::from_raw(node);
                }
            }
        }
    }

    pub unsafe fn refresh(&mut self) {
        let mut curr = self.list_head;
        while let Some(mut cnode) = curr {
            let node = cnode.as_mut();
            curr = node.next;
            let _ = *Box::from_raw(node);
        }
        self.list_head = None;
    }

    pub fn set_gc_duration(&mut self, duration: Duration) {
        self.gc_duration = duration;
    }
}

// This is the actual GC
thread_local!(pub static GC_STATE: RefCell<GcState> = RefCell::new(GcState::new()));

impl<T: Trace> GcNode<T> {
    pub fn new(val: T) -> NonNull<Self> {
        GC_STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.try_collect_garbage();
            let ptr = Box::into_raw(Box::new(GcNode {
                data: Cell::new(GcData::new()),
                next: state.list_head.take(),
                val,
            }));

            // SAFETY: box guaranteed to be non null (same for both)
            state.list_head = Some(unsafe { NonNull::new_unchecked(ptr) });
            unsafe { NonNull::new_unchecked(ptr) }
        })
    }
}

pub fn set_gc_duration(duration: Duration) {
    GC_STATE.with(|state| {
        state.borrow_mut().set_gc_duration(duration);
    });
}
