use crate::trace_derive;
use synstructure::test_derive;

#[test]
fn test() {
    test_derive!{
        trace_derive {
            struct A {
                a: u32,
                b: u32,
            }
        }
        expands to {
            impl gc_rs::Trace for A {
                fn trace(&self) {
                    match self {
                        A { a, b } => {
                            ::gc_rs::Trace::trace(a);
                            ::gc_rs::Trace::trace(b);
                        }
                    }
                }
                fn root_children(&self) {
                    match self {
                        A { a, b } => {
                            ::gc_rs::Trace::root_children(a);
                            ::gc_rs::Trace::root_children(b);
                        }
                    }
                }
                fn deroot_children(&self) {
                    match self {
                        A { a, b } => {
                            ::gc_rs::Trace::deroot_children(a);
                            ::gc_rs::Trace::deroot_children(b);
                        }
                    }
                }
            }
        } no_build
    }
}
