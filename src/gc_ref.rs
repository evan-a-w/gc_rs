use crate::gc::*;
use crate::gc_obj::*;
use std::ops::{Deref, DerefMut};

pub struct GcRef<'a, T: Trace<T>> {
    pub gc_obj: &'a GcObj<T>,
}

pub struct GcUnique<T: Trace<T>> {
    pub gc_obj: GcObj<T>,
}

pub struct GcRefMut<'a, T: Trace<T>> {
    pub gc_obj: &'a GcObj<T>,
}

impl<'a, T: Trace<T>> GcRefMut<'a, T> {
    pub fn inner(&mut self) -> &mut T {
        unsafe { self.gc_obj.data.get().as_mut().unwrap().as_mut() }
    }
}

impl<'a, T: Trace<T>> GcRef<'a, T> {
    pub fn inner(&self) -> &T {
        unsafe { self.gc_obj.data.get().as_ref().unwrap().as_ref() }
    }
}

impl<'a, T: Trace<T>> Drop for GcRef<'a, T> {
    fn drop(&mut self) {
        self.gc_obj.remove_shared();
    }
}

impl<'a, T: Trace<T>> Drop for GcRefMut<'a, T> {
    fn drop(&mut self) {
        unsafe {
            self.gc_obj.flags.get().as_mut().unwrap().taken = TakenFlag::NotTaken;
        }
    }
}

impl<T: Trace<T>> Deref for GcRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.gc_obj.data.get().as_ref().unwrap().as_ref() }
    }
}

impl<T: Trace<T>> Deref for GcRefMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.gc_obj.data.get().as_ref().unwrap().as_ref() }
    }
}

impl<T: Trace<T>> DerefMut for GcRefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.gc_obj.data.get().as_mut().unwrap().as_mut() }
    }
}

impl<T: Trace<T>> DerefMut for GcUnique<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.gc_obj.data.get().as_mut().unwrap().as_mut() }
    }
}

impl<T: Trace<T>> Deref for GcUnique<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.gc_obj.data.get().as_ref().unwrap().as_ref() }
    }
}

impl<T: Trace<T>> Drop for GcUnique<T> {
    fn drop(&mut self) {
        unsafe {
            self.gc_obj.flags.get().as_mut().unwrap().taken = TakenFlag::NotTaken;
        }
    }
}
