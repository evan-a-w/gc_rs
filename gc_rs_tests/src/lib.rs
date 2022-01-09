use gc_rs::{Trace, Gc, GC_STATE};

#[cfg(test)]
mod tests {
    use crate::*;
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

    #[test]
    fn test_hashmap() {
        hashmap();
    }

    #[test]
    fn test_linked_list() {
        linked_list();
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

pub fn hashmap() {
    use std::collections::HashMap;
    use std::thread::sleep;
    use std::time::Duration;
    {
        let map = Gc::new(HashMap::new());

        for i in 0..500usize {
            let _ref = &*map;
            let mut_ref = &mut *map.borrow_mut().unwrap();
            mut_ref.insert(format!("{}", i), i);
            // Aliased - does this crash?
            assert!(_ref.get(&format!("{}", i)).unwrap() == &i);
        }

        sleep(Duration::from_millis(5000));

        for i in 500..1000 {
            map.borrow_mut().unwrap().insert(format!("{}", i), i);
        }

        GC_STATE.with(|st| st.borrow_mut().collect_garbage());

        for i in 0..1000 {
            assert!(*map.get(&format!("{}", i)).unwrap() == i);
            assert!(*map.get(&format!("{}", i)).unwrap() == i);
            assert!(*map.get(&format!("{}", i)).unwrap() == i);
        }

        assert!(*map.get("50").unwrap() == 50);
        assert!(*map.get("999").unwrap() == 999);
    }

    GC_STATE.with(|st| st.borrow_mut().collect_garbage());
}

pub fn linked_list() {
    #[derive(Trace, Clone, PartialEq)]
    struct LinkedList {
        pub val: i32,
        pub next: Option<Gc<LinkedList>>,
    }

    fn new(val: i32, next: Option<Gc<LinkedList>>) -> Gc<LinkedList> {
        Gc::new(LinkedList { val, next })
    }

    fn from_vec(vec: Vec<i32>) -> Option<Gc<LinkedList>> {
        let mut head = None;
        for i in vec.into_iter().rev() {
            head = Some(new(i, head));
        }
        head
    }

    fn reverse(list: Option<Gc<LinkedList>>) -> Option<Gc<LinkedList>> {
        let mut head = None;
        let mut curr = list;
        while let Some(l) = curr {
            head = Some(new(l.val, head));
            curr = l.next.clone();
        }
        head
    }

    assert!(from_vec(vec![1, 2, 3]) == reverse(from_vec(vec![3, 2, 1])));

    assert!(from_vec(vec![]) == None);

    assert!(reverse(from_vec(vec![])) == None);
}
