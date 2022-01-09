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

impl<T: Trace + ?Sized + 'static> Clone for Gc<T> {
    fn clone(&self) -> Self {
        let gc_node_ptr = self.gc_node_ptr.clone();
        let borrowed = self.borrowed.clone();
        let root = Cell::new(true);
        let res = Gc {
            gc_node_ptr,
            borrowed,
            root,
        };
        res.root();
        res
    }
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

    pub fn borrow_mut(&self) -> Option<GcRefMut<T>> {
        if self.borrowed.get() {
            return None;
        }
        self.borrowed.set(true);
        Some(GcRefMut { gc_node_ptr: self.gc_node_ptr, borrowed: self.borrowed.clone() })
    }


    pub fn is_root(&self) -> bool {
        self.root.get()
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.gc_node_ptr == other.gc_node_ptr
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
        // SAFETY: The value cannot be mutably borrowed. It can be immutably borrowed
        // actually but just dont :))
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

    fn root(&self) {
        if !self.root.get() {
            self.root.set(true);
            unsafe {
                let r = self.gc_node_ptr.as_ref();
                let mut data = r.data.get();
                data.add_roots();
                r.data.set(data);
            }
        }
    }

    fn deroot(&self) {
        if self.root.get() {
            self.root.set(false);
            unsafe {
                let r = self.gc_node_ptr.as_ref();
                let mut data = r.data.get();
                data.sub_roots();
                r.data.set(data);
            }
        }
    }
}

impl<T: std::fmt::Display + Trace> std::fmt::Display for Gc<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}

impl<T: PartialEq + Trace> PartialEq for Gc<T> {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}

impl<T: std::fmt::Debug + Trace> std::fmt::Debug for Gc<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Gc({:?})", self.deref())
    }
}

impl<T: Trace> Trace for Option<T> {
    fn trace(&self) {
        if let Some(ref val) = self {
            val.trace();
        }
    }

    fn root_children(&self) {
        if let Some(ref val) = self {
            val.root_children();
        }
    }

    fn deroot_children(&self) {
        if let Some(ref val) = self {
            val.deroot_children();
        }
    }

    fn root(&self) {
        if let Some(ref val) = self {
            val.root();
        }
    }

    fn deroot(&self) {
        if let Some(ref val) = self {
            val.deroot();
        }
    }
}

impl<T: Trace, E> Trace for Result<T, E> {
    fn trace(&self) {
        if let Ok(ref val) = self {
            val.trace();
        }
    }

    fn root_children(&self) {
        if let Ok(ref val) = self {
            val.root_children();
        }
    }

    fn deroot_children(&self) {
        if let Ok(ref val) = self {
            val.deroot_children();
        }
    }

    fn root(&self) {
        if let Ok(ref val) = self {
            val.root();
        }
    }

    fn deroot(&self) {
        if let Ok(ref val) = self {
            val.deroot();
        }
    }
}
