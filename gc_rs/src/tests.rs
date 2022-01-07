use std::ptr::NonNull;
use crate::gc::*;
use crate::gc_state::*;
use crate::traits::*;
use gc_rs_derive::Trace;

pub fn manual_trait() {
    struct Foo {
        pub x: i32,
        pub y: String,
    }

    impl Trace for Foo {
        fn trace(&self) {}

        fn root_children(&self) {}

        fn deroot_children(&self) {}
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

#[test]
fn test_manual_trait() {
    manual_trait();
}
