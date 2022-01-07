use crate::gc_obj::*;
use crate::gc_ref::*;
use std::cell::Cell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use std::ptr::NonNull;
use std::rc::Rc;


#[derive(Debug)]
pub struct Gc<T: Trace<T>> {
    pub ptrs: HashMap<usize, GcObj<T>>,
    pub max_id: usize,
    pub last_gc: Instant,
    pub gc_duration: Duration,
}

impl<T: Trace<T>> Gc<T> {
    pub fn new() -> Gc<T> {
        Gc {
            ptrs: HashMap::new(),
            max_id: 0,
            last_gc: Instant::now(),
            gc_duration: Duration::from_secs(5),
        }
    }

    pub fn try_collect_garbage(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_gc) > self.gc_duration {
            self.last_gc = now;
            self.collect_garbage();
        }
    }

    pub fn collect_garbage(&mut self) {
        for obj in self.ptrs.values() {
            match obj.get_flags().taken {
                TakenFlag::NotTaken => (),
                _ => { obj.trace(self); },
            }
        }

        let mut to_delete = vec![];
        for obj in self.ptrs.values() {
            match obj.get_flags().marker {
                MarkerFlag::Unseen => {
                    to_delete.push(obj.id);
                }
                _ => obj.mark_unseen(),
            }
        }

        for id in to_delete {
            // Drop is implemented already
            self.ptrs.remove(&id);
        }
    }

    pub fn take(&mut self, id: usize) -> Option<T> {
        self.try_collect_garbage();
        let obj = self.ptrs.get_mut(&id)?;
        // Ensure that can't be owned while refs are out
        if obj.get_flags().taken != TakenFlag::NotTaken {
            return None;
        }
        // Safety: Not taken, so ownership can be safely transferred
        let res = unsafe { *Box::from_raw(obj.get_ptr()) };
        self.ptrs.remove(&id);
        Some(res)
    }

    pub fn get_new_id(&mut self) -> usize {
        let id = self.max_id;
        self.max_id += 1;
        id
    }

    pub fn add(&mut self, data: T) -> usize {
        self.try_collect_garbage();
        let obj = GcObj::new(self, data);
        let id = obj.id;
        self.ptrs.insert(obj.id, obj);
        id
    }

    pub fn add_id(&mut self, data: T, id: usize) {
        self.try_collect_garbage();
        // Safety: Box is not null
        let flags = Rc::new(Cell::new(Flags {
            marker: MarkerFlag::Unseen,
            taken: TakenFlag::NotTaken,
            free: false,
        }));
        let refs = Rc::new(Cell::new(1));
        let data = unsafe {
            NonNull::new_unchecked(Box::into_raw(Box::new(data)))
        };
        let obj = GcObj::new_data(id, flags, refs, data);
        self.ptrs.insert(id, obj);
    }

    pub fn get_ref<'a>(&'a self, id: usize) -> Option<&'a T> {
        let val = self.ptrs.get(&id)?;
        let flag = val.get_flags();
        if flag.taken == TakenFlag::NotTaken {
            // Safety: Not taken, so can be safely read
            // (not really, because we can't keep track of borrow status)
            // Use at own peril.
            unsafe { Some(&*val.get_ptr()) }
        } else {
            None
        }
    }

    pub fn get_ref_mut<'a>(&'a self, id: usize) -> Option<&'a mut T> {
        let val = self.ptrs.get(&id)?;
        let flag = val.get_flags();
        if flag.taken == TakenFlag::NotTaken {
            // Safety: Not taken, so can be safely read
            // (not really, because we can't keep track of borrow status)
            // Use at own peril.
            unsafe { Some(&mut *val.get_ptr()) }
        } else {
            None
        }
    }

    pub fn get_gc_obj(&self, id: usize) -> Option<GcObj<T>> {
        let obj = self.ptrs.get(&id)?;
        Some(obj.clone())
    }

    pub fn get(&self, id: usize) -> Option<GcRef<T>> {
        let obj = self.ptrs.get(&id)?;
        obj.borrow()
    }

    pub fn get_mut(&self, id: usize) -> Option<GcRefMut<T>> {
        let obj = self.ptrs.get(&id)?;
        obj.borrow_mut()
    }
}

impl<T: Trace<T>> GcObj<T> {
    pub fn new(state: &mut Gc<T>, data: T) -> GcObj<T> {
        // Safety: Box is not null
        let data = unsafe {
            NonNull::new_unchecked(Box::into_raw(Box::new(data)))
        };
        let refs = Rc::new(Cell::new(1));
        let flags = Rc::new(Cell::new(Flags {
            marker: MarkerFlag::Unseen,
            taken: TakenFlag::NotTaken,
            free: false,
        }));
        GcObj::new_data(state.get_new_id(), flags, refs, data)
    }
}


// Should call trace recursively on children that are garbage collected.
// Should not modify data other than that.
pub trait Trace<T: Trace<T>> {
    fn trace(&self, gc: &Gc<T>);
}
