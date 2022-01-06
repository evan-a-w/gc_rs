use crate::gc_obj::*;
use crate::gc_ref::*;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use std::ptr::NonNull;
use std::rc::Rc;

pub struct Gc<T: Trace<T>> {
    pub ptrs: HashMap<usize, GcObj<T>>,
    pub max_id: usize,
    pub last_gc: Instant,
    pub gc_duration: Duration,
    pub roots: HashSet<usize>,
}

impl<T: Trace<T>> Gc<T> {
    pub fn new() -> Gc<T> {
        Gc {
            ptrs: HashMap::new(),
            max_id: 0,
            last_gc: Instant::now(),
            gc_duration: Duration::from_secs(5),
            roots: HashSet::new(),
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
                TakenFlag::NotTaken => { self.roots.insert(obj.id); },
                _ => (),
            }
        }

        for id in self.roots.iter() {
            self.get(*id).unwrap().trace(self);
        }

        let mut to_delete = vec![];
        for obj in self.ptrs.values() {
            match obj.get_marker() {
                MarkerFlag::Unseen => to_delete.push(obj.id),
                _ => obj.mark_unseen(),
            }
        }

        for id in to_delete {
            let gco = self.ptrs.get_mut(&id).unwrap();
            gco.free();
            self.ptrs.remove(&id);
        }
    }

    pub fn take(&mut self, id: usize) -> Option<T> {
        self.try_collect_garbage();
        let obj = self.ptrs.get_mut(&id)?;
        let res = unsafe { *Box::from_raw(obj.data.get().as_ref().unwrap().as_ptr()) };
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
        let obj = GcObj {
            data: UnsafeCell::new(
                NonNull::new(Box::into_raw(Box::new(data))).unwrap()
            ),
            flags: Rc::new(UnsafeCell::new(Flags {
                marker: MarkerFlag::Unseen,
                taken: TakenFlag::NotTaken,
            })),
            id,
        };
        self.ptrs.insert(id, obj);
    }

    pub fn get_unique(&mut self, id: usize) -> Option<GcUnique<T>> {
        let val = self.ptrs.get(&id)?;
        val.mark_taken()?;
        Some(GcUnique { gc_obj: val.clone() })
    }

    pub fn get_mut(&mut self, id: usize) -> Option<GcRefMut<T>> {
        let val = self.ptrs.get_mut(&id)?;
        val.mark_taken()?;
        Some(GcRefMut { gc_obj: val })
    }

    pub fn get(&self, id: usize) -> Option<GcRef<T>> {
        let val = self.ptrs.get(&id)?;
        val.add_shared()?;
        Some(GcRef { gc_obj: val })
    }
}

impl<T: Trace<T>> GcObj<T> {
    pub fn new(state: &mut Gc<T>, data: T) -> GcObj<T> {
        GcObj {
            data: UnsafeCell::new(
                NonNull::new(Box::into_raw(Box::new(data))).unwrap()
            ),
            flags: Rc::new(UnsafeCell::new(Flags {
                marker: MarkerFlag::Unseen,
                taken: TakenFlag::NotTaken,
            })),
            id: state.get_new_id(),
        }
    }
}

pub trait Trace<T: Trace<T>> {
    fn trace(&self, gc: &Gc<T>);
}


// Just calls the true trace on everything that could be a Ref
impl<T: Trace<T>> Drop for Gc<T> {
    fn drop(&mut self) {
        for obj in self.ptrs.values_mut() {
            obj.free();
        }
    }
}
