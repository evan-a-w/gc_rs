use gc_rs::gc::{Gc, Trace};
use gc_rs::gc_obj::GcObj;
use gc_rs::gc_ref::{GcRef, GcRefMut};

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

fn gc_ref_test() {
    let mut gc: Gc<Data> = Gc::new();

    let first_id;
    {
        let first = Data {
            s: "first".to_string(),
            inner: None,
        };

        first_id = gc.add(first);
        let first_ref = gc.get(first_id).unwrap();

        gc.collect_garbage();


        let inner = gc.get(first_id);
        let second = Data {
            s: "second".to_string(),
            inner,
        };

        let second_id = gc.add(second);
        let second_ref = gc.get(second_id).unwrap();

        gc.collect_garbage();
        // No garbage should be collected yet

        // Moves out second_ref - it will be dropped after this if we collect garbage
        inner_function(second_ref, &mut gc);

        // Should free the second_ref data, but not first ref
        gc.collect_garbage();

        assert!(gc.ptrs.len() == 1);
        assert!(first_ref.s == "first");
    }

    {
        let first_obj = gc.get_gc_obj(first_id).unwrap();

        {
            let mut borrow_mut = first_obj.borrow_mut().unwrap();
            borrow_mut.s = "first_mut".to_string();
            assert!(first_obj.borrow_mut().is_none());
        }

        {
            let borrow = first_obj.borrow().unwrap();
            assert!(borrow.s == "first_mut".to_string());
            assert!(first_obj.borrow().is_some());
        }
    }
}

fn main() {
    gc_ref_test();
}
