use std::rc::Rc;

use dioxus_core::prelude::{has_context, provide_context};
use generational_box::{GenerationalBox, Owner, Storage};

pub fn current_owner<T, S: Storage<dyn Source<T>>>() -> Rc<Owner<S>> {
    match has_context() {
        Some(rt) => rt,
        None => {
            let owner = Rc::new(S::owner());
            provide_context(owner)
        }
    }
}

pub struct ReadOnly;
pub struct Writable;
pub struct Untracked;

pub trait SupportsWrites {}
impl SupportsWrites for Writable {}
impl SupportsWrites for Untracked {}

// Tracks reads/writes
pub struct SignalSource<T: 'static> {
    pub value: T,
}

pub struct SourceHolder<T: 'static> {
    pub t: T,
}

pub trait Source<T>: 'static {
    /// Read the value
    fn read(&self) -> &T;

    // Write the value
    fn write(&mut self) -> &mut T;

    /// Mark this value as read
    fn tracked_read(&self) {}

    /// Mark this value as written
    fn tracked_write(&mut self) {}
}

impl<T> Source<T> for SignalSource<T> {
    fn read(&self) -> &T {
        &self.value
    }
    fn write(&mut self) -> &mut T {
        &mut self.value
    }
}
