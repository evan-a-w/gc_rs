pub mod gc;
pub mod gc_obj;
pub mod gc_ref;

// Usage: the garbage collection requires a Gc<T> type to be passed around.
// Ownership can be shared by using GcObj (functions like Rc<RefCell<T>>),
// or &s can be directly used through GcRef, and &muts can be directly used
// through GcRefMut. Using any of these will prevent memory from being deleted.
