use gc_rs_tests::*;
use gc_rs::{Trace, Gc, GC_STATE};

fn main() {
    manual_trait();

    auto_trait();
}
