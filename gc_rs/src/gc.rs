use crate::traits::*;
use crate::gc_state::*;
use std::cell::{Cell, RefCell};
use std::ptr::NonNull;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

pub struct Gc<T: Trace + ?Sized + 'static> {
    gc_node_ptr: NonNull<GcNode<T>>,
    borrowed: Rc<Cell<bool>>,
    root: Cell<bool>,
}

pub struct GcRefMut<T: Trace + ?Sized + 'static> {
    gc_node_ptr: NonNull<GcNode<T>>,
    borrowed: Rc<Cell<bool>>,
}

impl<T: Trace> Gc<T> {
    pub fn new(value: T) -> Self {
        let val = GcNode::new(value);
        // Safety: Inaccessible elsewhere since it has just been created in the Gc
        unsafe {
            let r = val.as_ref();
            r.val.deroot_children();
        }
        Self {
            gc_node_ptr: val,
            borrowed: Rc::new(Cell::new(false)),
            root: Cell::new(true),
        }
    }

    pub unsafe fn get_roots(&self) -> usize {
        let r = self.gc_node_ptr.as_ref();
        r.data.get().get_roots()
    }

    pub fn borrow_mut(&mut self) -> Option<GcRefMut<T>> {
        if self.borrowed.get() {
            return None;
        }
        self.borrowed.set(true);
        Some(GcRefMut { gc_node_ptr: self.gc_node_ptr, borrowed: self.borrowed.clone() })
    }

    pub fn root(&self) {
        self.root.set(true);
        unsafe {
            let r = self.gc_node_ptr.as_ref();
            let mut data = r.data.get();
            data.add_roots();
            r.data.set(data);
        }
    }

    pub fn deroot(&self) {
        self.root.set(false);
        unsafe {
            let r = self.gc_node_ptr.as_ref();
            let mut data = r.data.get();
            data.sub_roots();
            r.data.set(data);
        }
    }

    pub fn is_root(&self) -> bool {
        self.root.get()
    }
}

impl<T: Trace + ?Sized + 'static> Deref for Gc<T> {
    type Target = T;
    // Panics if mutably borrowed
    fn deref(&self) -> &Self::Target {
        if self.borrowed.get() {
            panic!("Cannot deref a Gc that is mutably borrowed");
        }
        // SAFETY: The value cannot be mutably borrowed (panics if so)
        unsafe { &self.gc_node_ptr.as_ref().val }
    }
}

impl<T: Trace + ?Sized + 'static> Deref for GcRefMut<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: The value cannot be mutably borrowed (GcRefMut guarantees such)
        unsafe { &self.gc_node_ptr.as_ref().val }
    }
}

impl<T: Trace + ?Sized + 'static> DerefMut for GcRefMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: The value cannot be mutably borrowed or immutably borrowed (GcRefMut guarantees such)
        unsafe { &mut self.gc_node_ptr.as_mut().val }
    }
}

impl<T: Trace + ?Sized + 'static> Drop for GcRefMut<T> {
    fn drop(&mut self) {
        self.borrowed.set(false);
    }
}

impl<T: Trace + ?Sized + 'static> Drop for Gc<T> {
    fn drop(&mut self) {
        if self.root.get() {
            unsafe {
                let r = self.gc_node_ptr.as_mut();
                r.data.get_mut().sub_roots();
                if !r.data.get().is_root() {
                    self.root.set(false);
                }
            }
        }
    } 
}

impl<T: Trace + ?Sized + 'static> Trace for Gc<T> {
    fn trace(&self) {
        unsafe { 
            let ptr = self.gc_node_ptr.as_ref();
            let mut data = ptr.data.get();
            if !data.is_marked() {
                data.mark();
                ptr.data.set(data);
                ptr.val.trace();
            }
        }
    }

    fn root_children(&self) {
        unsafe { 
            let ptr = self.gc_node_ptr.as_ref();
            ptr.val.root_children();
        }
    }

    fn deroot_children(&self) {
        unsafe { 
            let ptr = self.gc_node_ptr.as_ref();
            ptr.val.deroot_children();
        }
    }
}
