use std::rc::Rc;

pub trait Trace {
    // Trace calls trace on all Gc children and marks them
    fn trace(&self);

    fn root_children(&self);

    fn deroot_children(&self);
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
    };
}

macro_rules! simple_empty_finalize_trace {
    ($($T:ty),*) => {
        $(
            impl Trace for $T { empty_trace!(); }
        )*
    }
}

simple_empty_finalize_trace![
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
