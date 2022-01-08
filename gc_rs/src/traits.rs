use std::rc::Rc;

pub trait Trace {
    // Trace calls trace on all Gc children and marks them
    fn trace(&self);

    fn root_children(&self);

    fn deroot_children(&self);

    fn root(&self);

    fn deroot(&self);
}

#[macro_export]
macro_rules! empty_trace {
    () => {
        #[inline]
        fn trace(&self) {}
        #[inline]
        fn root_children(&self) {}
        #[inline]
        fn deroot_children(&self) {}
        #[inline]
        fn root(&self) {}
        #[inline]
        fn deroot(&self) {}
    };
}

#[macro_export]
macro_rules! iter_trace {
    () => {
        #[inline]
        fn trace(&self) {
            for item in self {
                item.trace();
            }
        }

        #[inline]
        fn root_children(&self) {
            for item in self {
                item.root_children();
            }
        }

        #[inline]
        fn deroot_children(&self) {
            for item in self {
                item.deroot_children();
            }
        }

        #[inline]
        fn root(&self) {}

        #[inline]
        fn deroot(&self) {}
    };
}

macro_rules! simple_empty_trace {
    ($($T:ty),*) => {
        $(
            impl Trace for $T { empty_trace!(); }
        )*
    }
}

macro_rules! simple_iter_trace {
    ($($T:ty),*) => {
        $(
            impl<X: Trace> Trace for $T {
                iter_trace!();
            }
        )*
    }
}

simple_empty_trace![
    (),
    bool,
    isize,
    usize,
    i8,
    u8,
    i16,
    u16,
    i32,
    u32,
    i64,
    u64,
    i128,
    u128,
    f32,
    f64,
    char,
    String,
    Box<str>,
    Rc<str>
];

simple_iter_trace![
    Vec<X>,
    std::collections::LinkedList<X>
];

impl<T, X: Trace> Trace for std::collections::HashMap<T, X> {
    #[inline]
    fn trace(&self) {
        for item in self.values() {
            item.trace();
        }
    }

    #[inline]
    fn root_children(&self) {
        for item in self.values() {
            item.root_children();
        }
    }

    #[inline]
    fn deroot_children(&self) {
        for item in self.values() {
            item.deroot_children();
        }
    }

    #[inline]
    fn root(&self) {}

    #[inline]
    fn deroot(&self) {}
}
