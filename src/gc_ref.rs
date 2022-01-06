use crate::gc::*;
use crate::gc_obj::*;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::cell::Cell;
use std::rc::Rc;
use std::convert::{TryInto, Into, AsRef, AsMut};

#[derive(Debug)]
pub struct GcRef<T: Trace<T>> {
    ptr: NonNull<T>,
    flags: Rc<Cell<Flags>>,
}

#[derive(Debug)]
pub struct GcRefMut<T: Trace<T>> {
    ptr: NonNull<T>,
    flags: Rc<Cell<Flags>>,
}

impl<T: Trace<T>> GcRef<T> {
    pub fn new(ptr: NonNull<T>, flags: Rc<Cell<Flags>>) -> Self {
        GcRef {
            ptr,
            flags,
        }
    }
}

impl<T: Trace<T>> GcRefMut<T> {
    pub fn new(ptr: NonNull<T>, flags: Rc<Cell<Flags>>) -> Self {
        GcRefMut {
            ptr,
            flags,
        }
    }
}

impl<T: Trace<T>> Deref for GcRef<T> {
    type Target = T;

    fn deref(&self) -> &T {
        // Safety - guaranteed not to be uniquely borrowed through borrow flags
        // so no mutable borrows can occur
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: Trace<T>> Deref for GcRefMut<T> {
    type Target = T;

    fn deref(&self) -> &T {
        // Safety - guaranteed to be uniquely borrowed through borrow flags
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: Trace<T>> DerefMut for GcRefMut<T> {
    fn deref_mut(&mut self) -> &mut T {
        // Safety - guaranteed to be uniquely borrowed through borrow flags
        unsafe { self.ptr.as_mut() }
    }
}

impl<T: Trace<T>> AsRef<T> for GcRef<T> {
    fn as_ref(&self) -> &T {
        // Safety - guaranteed to not be uniquely borrowed or owned through
        // flags.
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: Trace<T>> AsRef<T> for GcRefMut<T> {
    fn as_ref(&self) -> &T {
        // Safety - guaranteed to not be uniquely borrowed or owned through
        // flags.
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: Trace<T>> AsMut<T> for GcRefMut<T> {
    fn as_mut(&mut self) -> &mut T {
        // Safety - guaranteed to be uniqued borrowed by flags
        unsafe { self.ptr.as_mut() }
    }
}

impl<T: Trace<T>> Drop for GcRef<T> {
    fn drop(&mut self) {
        let mut flags = self.flags.get();
        flags.remove_shared();
        if flags.free && flags.taken == TakenFlag::NotTaken {
            // Safety: Free only if there are now no references - no double free
            unsafe {
                let _ = *Box::from_raw(self.ptr.as_ptr());
            }
        }
        self.flags.set(flags);
    }
}

impl<T: Trace<T>> Drop for GcRefMut<T> {
    fn drop(&mut self) {
        let mut flags = self.flags.get();
        flags.remove_shared();
        let flags = self.flags.get();
        // Safety: Since its a unique reference, there can't be others, so we
        // can free data. If the flag isn't unique, it means that it has been
        // converted into a GcRef, which means we don't want to free it
        // (transferred ownership to the GcRef)
        if flags.free && flags.taken == TakenFlag::Unique {
            unsafe {
                let _ = *Box::from_raw(self.ptr.as_ptr());
            }
        }
        self.flags.set(flags);
    }
}

impl<T: Trace<T>> TryInto<GcRefMut<T>> for GcRef<T> {
    type Error = ();

    fn try_into(self) -> Result<GcRefMut<T>, Self::Error> {
        let mut flags = self.flags.get();
        match flags.taken {
            TakenFlag::Shared(1) => {
                flags.taken = TakenFlag::Unique;
            }
            _ => {
                return Err(());
            }
        }
        self.flags.set(flags);
        Ok(GcRefMut::new(self.ptr, self.flags.clone()))
    }
}

impl<T: Trace<T>> Into<GcRef<T>> for GcRefMut<T> {
    fn into(self) -> GcRef<T> {
        let mut flags = self.flags.get();
        flags.taken = TakenFlag::Shared(1);
        self.flags.set(flags);
        GcRef::new(self.ptr, self.flags.clone())
    }
}

impl<T: Trace<T>> Trace<T> for GcRef<T> {
    fn trace(&self, gc: &Gc<T>) {
        let mut flags = self.flags.get();
        if flags.marker == MarkerFlag::Seen {
            return;
        }
        flags.mark_children_not_seen();
        self.as_ref().trace(gc);
        flags.mark_seen();
        self.flags.set(flags);
    }
}

impl<T: Trace<T>> Trace<T> for GcRefMut<T> {
    fn trace(&self, gc: &Gc<T>) {
        let mut flags = self.flags.get();
        if flags.marker == MarkerFlag::Seen {
            return;
        }
        flags.mark_children_not_seen();
        self.as_ref().trace(gc);
        flags.mark_seen();
        self.flags.set(flags);
    }
}

impl<T: Trace<T>> Clone for GcRef<T> {
    fn clone(&self) -> Self {
        let mut flags = self.flags.get();
        flags.taken = TakenFlag::Shared(1);
        self.flags.set(flags);
        GcRef::new(self.ptr, self.flags.clone())
    }
}
