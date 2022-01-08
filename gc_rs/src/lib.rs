#![feature(log_syntax)]

pub mod gc_state;
pub mod traits;
pub mod gc;

pub use gc_rs_derive::Trace;

pub use gc::{Gc, GcRefMut};

pub use gc_state::{set_gc_duration, GC_STATE};

pub use traits::Trace;
