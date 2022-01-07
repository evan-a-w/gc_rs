use std::cell::Cell;
use std::ops::{Deref, DerefMut};
use crate::gc::Trace;
use std::ptr::NonNull;
use crate::gc::*;
use crate::gc_ref::*;
use std::rc::Rc;

// Reference counted access of garbage collected value.
// Acts like Rc<RefCell<T>> - can borrow and borrow_mut
#[derive(Debug)]
pub struct GcObj<T: Trace<T>> {
    pub id: usize,
    flags: Rc<Cell<Flags>>,
    refs: Rc<Cell<usize>>,
    data: NonNull<T>,
}

impl<T: Trace<T>> Clone for GcObj<T> {
    fn clone(&self) -> Self {
        let mut refs = self.refs.get();
        refs += 1;
        self.refs.set(refs);
        GcObj {
            id: self.id,
            flags: self.flags.clone(),
            refs: self.refs.clone(),
            data: self.data,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MarkerFlag {
    Unseen,
    ChildrenNotSeen,
    Seen,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TakenFlag {
    NotTaken,
    Shared(usize),
    Unique,
}

#[derive(Clone, Copy, Debug)]
pub struct Flags {
    pub marker: MarkerFlag,
    pub taken: TakenFlag,
    pub free: bool,
}

impl Flags {
    pub fn add_shared(&mut self) -> Option<()> {
        match self.taken {
            TakenFlag::NotTaken => {
                self.taken = TakenFlag::Shared(1);
                Some(())
            }
            TakenFlag::Shared(n) => {
                self.taken = TakenFlag::Shared(n + 1);
                Some(())
            }
            _ => None,
        }
    }

    pub fn remove_shared(&mut self) -> Option<()> {
        match self.taken {
            TakenFlag::Shared(n) => {
                if n == 1 {
                    self.taken = TakenFlag::NotTaken;
                } else {
                    self.taken = TakenFlag::Shared(n - 1);
                }
                Some(())
            }
            _ => None,
        }
    }

    pub fn mark_seen(&mut self) {
        self.marker = MarkerFlag::Seen;
    }

    pub fn remove_taken(&mut self) {
        self.taken = TakenFlag::NotTaken;
    }

    pub fn mark_taken(&mut self) -> Option<()> {
        if self.taken == TakenFlag::NotTaken {
            self.taken = TakenFlag::Unique;
            Some(())
        } else {
            None
        }
    }

    pub fn mark_children_not_seen(&mut self) {
        self.marker = MarkerFlag::ChildrenNotSeen;
    }
}

impl<T: Trace<T>> GcObj<T> {
    pub fn new_data(id: usize, flags: Rc<Cell<Flags>>, refs: Rc<Cell<usize>>,
               data: NonNull<T>) -> Self {
        GcObj {
            id,
            flags,
            refs,
            data,
        }
    }

    pub fn get_flags(&self) -> Flags {
        self.flags.get()
    }

    pub fn get_refs(&self) -> usize {
        self.refs.get()
    }

    pub fn get_ptr(&self) -> *mut T {
        self.data.as_ptr()
    }
    
    pub fn add_shared(&self) -> Option<()> {
        let mut flag = self.flags.get();
        flag.add_shared()?;
        self.flags.set(flag);
        Some(())
    }

    pub fn remove_shared(&self) -> Option<()> {
        let mut flag = self.flags.get();
        flag.remove_shared()?;
        self.flags.set(flag);
        Some(())
    }

    pub fn mark_seen(&self) {
        let mut flag = self.flags.get();
        flag.mark_seen();
        self.flags.set(flag);
    }

    pub fn remove_taken(&self) {
        let mut flag = self.flags.get();
        flag.remove_taken();
        self.flags.set(flag);
    }

    pub fn mark_taken(&self) -> Option<()> {
        let mut flag = self.flags.get();
        flag.mark_taken()?;
        self.flags.set(flag);
        Some(())
    }

    pub fn mark_unseen(&self) {
        let mut flags = self.flags.get();
        flags.marker = MarkerFlag::Unseen;
        self.flags.set(flags);
    }

    pub fn mark_children_not_seen(&self) {
        let mut flag = self.flags.get();
        flag.mark_children_not_seen();
        self.flags.set(flag);
    }

    pub fn borrow(&self) -> Option<GcRef<T>> {
        self.add_shared()?;
        Some(GcRef::new(self.data, self.flags.clone()))
    }

    pub fn borrow_mut(&self) -> Option<GcRefMut<T>> {
        self.mark_taken()?;
        Some(GcRefMut::new(self.data, self.flags.clone()))
    }

    pub unsafe fn free(&mut self) {
        let _ = *Box::from_raw(self.data.as_ptr());
    }
}

// Marks as seen and calls trace on children
impl<T: Trace<T>> Trace<T> for GcObj<T> {
    fn trace(&self, gc: &Gc<T>) {
        // Probably don't need this variant
        let marker = self.flags.get().marker;
        if marker == MarkerFlag::Seen {
            return;
        }
        self.mark_children_not_seen();
        match self.borrow() {
            Some(ref gc_ref) => gc_ref.trace(gc),
            // Safety: Only modifies values in Cell. This is probably not fine
            // but I can't think of a better way to do it
            None => unsafe {
                let gc_ref = &*self.data.as_ptr();
                gc_ref.trace(gc);
            },
        };
        self.mark_seen();
    }
}

impl<T: Trace<T>> Drop for GcObj<T> {
    fn drop(&mut self) {
        let mut refs = self.refs.get();
        refs -= 1;
        self.refs.set(refs);
        if refs == 0 {
            let mut flags = self.flags.get();
            // Don't free if it's taken
            if flags.taken != TakenFlag::NotTaken {
                // This flag ensures that the memory will be freed (once) by the 
                // GcRefs or GcRefMut that have/has the data
                flags.free = true;
                self.flags.set(flags);
            } else {
                // Safety: Only frees if the value is not used elsewhere
                // (tracked through flags and ref count)
                unsafe { self.free(); }
            }
        }
    }
}
