#![feature(test)]

extern crate test;

use gc_rs::{Trace, Gc, GC_STATE};

#[cfg(test)]
mod tests {
    use crate::*;
    use test::Bencher;

    #[test]
    fn test_manual_trait() {
        manual_trait();
    }

    #[test]
    fn test_auto_trait() {
        auto_trait();
    }

    #[test]
    fn test_vec() {
        vec();
    }

    #[bench]
    fn bench_collection(b: &mut Bencher) {
        #[derive(Trace)]
        struct Foo {
            pub x: i32,
            pub y: String,
        }

        #[derive(Trace)]
        struct Bar {
            pub x: i32,
            pub y: Gc<Foo>,
        }
        
        b.iter(|| {
            {
                let mut v = Vec::new();
                for _ in 0..100000 {
                    let foo = Foo {
                        x: 1,
                        y: "hello".to_string(),
                    };
                    let bar = Bar {
                        x: 2,
                        y: Gc::new(foo),
                    };
                    v.push(Gc::new(bar));
                }
            }
            GC_STATE.with(|st| st.borrow_mut().collect_garbage());
            let mut len: usize = 0;
            let mut roots: usize = 0;
            GC_STATE.with(|st| {
                len = unsafe { st.borrow().get_ptrs_len() };
                roots = unsafe { st.borrow().get_roots_len() };
            });
            assert!(len == 0);
            assert!(roots == 0);
        });
    }
}

pub fn manual_trait() {
    struct Foo {
        pub x: i32,
        pub y: String,
    }

    impl Trace for Foo {
        fn trace(&self) {}

        fn root_children(&self) {}

        fn deroot_children(&self) {}

        fn root(&self) {}

        fn deroot(&self) {}
    }

    {
        let first = Gc::new(Foo { x: 1, y: "hi".to_string(), });
        GC_STATE.with(|st| st.borrow_mut().collect_garbage());
        let mut len: usize = 0;
        let mut roots: usize = 0;
        GC_STATE.with(|st| {
            len = unsafe { st.borrow().get_ptrs_len() };
            roots = unsafe { st.borrow().get_roots_len() };
        });
        assert!(first.is_root());
        assert!(len == 1);
        assert!(roots == 1);
    }

    GC_STATE.with(|st| st.borrow_mut().collect_garbage());

    assert!(GC_STATE.with(|st| {
        let len = unsafe { st.borrow().get_ptrs_len() };
        len
    }) == 0);

    struct Bar {
        pub x: i32,
        pub y: Gc<Foo>,
    }

    impl Trace for Bar {
        fn trace(&self) {
            self.y.trace();
        }

        fn root_children(&self) {
            self.y.root();
        }

        fn deroot_children(&self) {
            self.y.deroot();
        }

        fn root(&self) {}

        fn deroot(&self) {}
    }

    {
        let first = Gc::new(Foo { x: 1, y: "hi".to_string(), });
        let second = Gc::new(Bar { x: 2, y: first });
        assert!(!second.y.is_root());
        let mut len: usize = 0;
        let mut roots: usize = 0;
        GC_STATE.with(|st| {
            len = unsafe { st.borrow().get_ptrs_len() };
            roots = unsafe { st.borrow().get_roots_len() };
        });
        assert!(len == 2);
        assert!(roots == 1);
    }

    GC_STATE.with(|st| st.borrow_mut().collect_garbage());

    assert!(GC_STATE.with(|st| {
        let len = unsafe { st.borrow().get_ptrs_len() };
        len
    }) == 0);

    // test spammy allocation
    for _ in 0..1000 {
        let _ = Gc::new(Foo { x: 1, y: "hi".to_string(), });
    }

    GC_STATE.with(|st| st.borrow_mut().collect_garbage());
    assert!(GC_STATE.with(|st| {
        let len = unsafe { st.borrow().get_ptrs_len() };
        len
    }) == 0);
}

pub fn auto_trait() {
    #[derive(Trace)]
    struct Foo {
        pub x: i32,
        pub y: String,
    }

    {
        let first = Gc::new(Foo { x: 1, y: "hi".to_string(), });
        GC_STATE.with(|st| st.borrow_mut().collect_garbage());
        let mut len: usize = 0;
        let mut roots: usize = 0;
        GC_STATE.with(|st| {
            len = unsafe { st.borrow().get_ptrs_len() };
            roots = unsafe { st.borrow().get_roots_len() };
        });
        assert!(first.is_root());
        assert!(len == 1);
        assert!(roots == 1);
    }

    GC_STATE.with(|st| st.borrow_mut().collect_garbage());

    assert!(GC_STATE.with(|st| {
        let len = unsafe { st.borrow().get_ptrs_len() };
        len
    }) == 0);

    #[derive(Trace)]
    struct Bar {
        pub x: i32,
        pub y: Gc<Foo>,
    }

    {
        let first = Gc::new(Foo { x: 1, y: "hi".to_string(), });
        let second = Gc::new(Bar { x: 2, y: first });
        assert!(!second.y.is_root());
        let mut len: usize = 0;
        let mut roots: usize = 0;
        GC_STATE.with(|st| {
            len = unsafe { st.borrow().get_ptrs_len() };
            roots = unsafe { st.borrow().get_roots_len() };
        });
        assert!(len == 2);
        assert!(roots == 1);
    }

    GC_STATE.with(|st| st.borrow_mut().collect_garbage());

    assert!(GC_STATE.with(|st| {
        let len = unsafe { st.borrow().get_ptrs_len() };
        len
    }) == 0);

    // test spammy allocation
    for _ in 0..1000 {
        let _ = Gc::new(Foo { x: 1, y: "hi".to_string(), });
    }

    GC_STATE.with(|st| st.borrow_mut().collect_garbage());
    assert!(GC_STATE.with(|st| {
        let len = unsafe { st.borrow().get_ptrs_len() };
        len
    }) == 0);
}

pub fn vec() {
    #[derive(Trace)]
    struct Foo {
        pub x: i32,
        pub y: String,
    }

    {
        let mut v = Vec::new();
        v.push(Gc::new(Foo { x: 1, y: "hi".to_string() }));
    }

    GC_STATE.with(|st| st.borrow_mut().collect_garbage());
}
