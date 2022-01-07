pub mod gc_state;
pub mod traits;
pub mod gc;
pub mod tests;

#[cfg(feature = "derive")]
pub use gc_rs_derive::Trace;

pub use gc::{Gc, GcRefMut};

pub use gc_state::set_gc_duration;

pub use traits::Trace;
