use std::cell::Cell;
use std::ops::{Deref, DerefMut};
use crate::gc::Trace;
use std::ptr::NonNull;
use crate::gc::*;
use crate::gc_ref::*;
use std::rc::Rc;

#[derive(Debug)]
pub struct GcObj<T: Trace<T>> {
    pub id: usize,
    pub flags: Rc<Cell<Flags>>,
    pub data: NonNull<T>,
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

    pub fn free(&mut self) {
        // Safety: up to the caller to not double free - this is just for ease
        // of use
        unsafe {
            let _ = *Box::from_raw(self.data.as_ptr());
        }
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
        self.borrow().unwrap().trace(gc);
        self.mark_seen();
    }
}
