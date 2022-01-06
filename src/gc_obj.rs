use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use crate::gc::Trace;
use std::ptr::NonNull;
use crate::gc::*;
use crate::gc_ref::*;
use std::rc::Rc;

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
}

#[derive(Debug)]
pub struct GcObj<T: Trace<T>> {
    pub data: UnsafeCell<NonNull<T>>,
    pub id: usize,
    pub flags: Rc<UnsafeCell<Flags>>,
}

impl<T: Trace<T>> Clone for GcObj<T> {
    fn clone(&self) -> Self {
        GcObj {
            data: unsafe {
                UnsafeCell::new(NonNull::new_unchecked(
                    self.data.get().as_ref().unwrap().as_ptr()
                ))
            },
            id: self.id,
            flags: self.flags.clone(),
        }
    }
}


impl<T: Trace<T>> Deref for GcObj<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.data.get().as_ref().unwrap().as_ref() }
    }
}

impl<T: Trace<T>> DerefMut for GcObj<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.data.get().as_mut().unwrap().as_mut() }
    }
}

impl<T: Trace<T>> GcObj<T> {
    pub fn add_shared(&self) -> Option<()> {
        unsafe {
            let flags = &mut *self.flags.get();
            match flags.taken {
                TakenFlag::NotTaken => {
                    flags.taken = TakenFlag::Shared(1);
                    Some(())
                }
                TakenFlag::Shared(n) => {
                    flags.taken = TakenFlag::Shared(n + 1);
                    Some(())
                }
                _ => None,
            }
        }
    }

    pub fn remove_shared(&self) {
        unsafe {
            let flags = &mut *self.flags.get();
            match flags.taken {
                TakenFlag::Shared(n) => {
                    if n == 1 {
                        flags.taken = TakenFlag::NotTaken;
                    } else {
                        flags.taken = TakenFlag::Shared(n - 1);
                    }
                }
                _ => panic!("Trying to dec shared when it is not shared"),
            }
        }
    }

    pub fn mark_seen(&self) {
        unsafe {
            let flags = &mut *self.flags.get();
            flags.marker = MarkerFlag::Seen;
        }
    }

    pub fn remove_taken(&self) {
        unsafe {
            let flags = &mut *self.flags.get();
            flags.taken = TakenFlag::NotTaken;
        }
    }

    pub fn mark_taken(&self) -> Option<()> {
        unsafe {
            let flags = &mut *self.flags.get();
            if flags.taken == TakenFlag::NotTaken {
                flags.taken = TakenFlag::Unique;
                Some(())
            } else {
                None
            }
        }
    }

    pub fn mark_unseen(&self) {
        unsafe {
            let flags = &mut *self.flags.get();
            flags.marker = MarkerFlag::Unseen;
        }
    }

    pub fn mark_children_not_seen(&self) {
        unsafe {
            let flags = &mut *self.flags.get();
            flags.marker = MarkerFlag::ChildrenNotSeen;
        }
    }

    pub fn borrow<'a>(&'a self) -> Option<GcRef<'a, T>> {
        unsafe {
            let flags = &mut *self.flags.get();
            match flags.taken {
                TakenFlag::Unique => None,
                _ => {
                    self.add_shared()?;
                    Some(GcRef {
                        gc_obj: self,
                    })
                }
            }
        }
    }

    pub fn borrow_mut<'a>(&'a self) -> Option<GcRefMut<'a, T>> {
        unsafe {
            let flags = &mut *self.flags.get();
            match flags.taken {
                TakenFlag::NotTaken => {
                    flags.taken = TakenFlag::Unique;
                    Some(GcRefMut {
                        gc_obj: &*(self as *const GcObj<T>),
                    })
                }
                _ => None,
            }
        }
    }

    pub fn get_flags(&self) -> Flags {
        unsafe { *self.flags.get() }
    }

    pub fn get_marker(&self) -> MarkerFlag {
        unsafe {
            let flags = &*self.flags.get();
            flags.marker
        }
    }

    pub fn free(&mut self) {
        unsafe {
            let _ = *Box::from_raw(self.data.get().as_ref().unwrap().as_ptr());
        }
    }
}

// Marks as seen and calls trace on children
impl<T: Trace<T>> Trace<T> for GcObj<T> {
    fn trace(&self, gc: &Gc<T>) {
        // Probably don't need this variant
        self.mark_children_not_seen();

        self.borrow().unwrap().trace(gc);
        self.mark_seen();
    }
}
