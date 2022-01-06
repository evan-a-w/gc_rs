mod gc;
mod gc_obj;
mod gc_ref;

use crate::gc::{Gc, Trace};
use crate::gc_obj::GcObj;
use crate::gc_ref::{GcRef, GcRefMut};

// Tests are ran here so valgrind can just be used


#[derive(Debug)]
struct Data {
    pub s: String,
    pub inner: Option<GcRef<Data>>,
}

// Need to make Trace impl - just call recursively on refs.
// Might be able to make derivable
impl Trace<Data> for Data {
    fn trace(&self, gc: &Gc<Data>) {
        match self.inner {
            Some(ref inner) => inner.trace(gc),
            None => (),
        }
    }
}

fn inner_function(data: GcRef<Data>, gc: &mut Gc<Data>) {
    assert!(data.inner.as_ref().unwrap().s == "first");
    for _ in 0..100 {
        gc.add(Data { s: "pointless".to_string(), inner: None, });
    }
}

fn main() {
    let mut gc: Gc<Data> = Gc::new();

    let first = Data {
        s: "first".to_string(),
        inner: None,
    };

    let first_id = gc.add(first);
    let first_ref = gc.get(first_id).unwrap();

    gc.collect_garbage();

    let second = Data {
        s: "second".to_string(),
        inner: gc.get(first_id),
    };

    let second_id = gc.add(second);
    let second_ref = gc.get(second_id).unwrap();

    gc.collect_garbage();
    // No garbage should be collected yet

    // Moves out second_ref - it will be dropped after this if we collect garbage
    inner_function(second_ref, &mut gc);

    // Should free the second_ref data, but not first ref
    gc.collect_garbage();

    println!("{}", gc.ptrs.len());
    assert!(first_ref.s == "first");
}
