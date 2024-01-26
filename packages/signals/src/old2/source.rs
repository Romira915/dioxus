use std::any::Any;

use generational_box::{AnyStorage, GenericStorage, Storage};

pub type UnsyncSignalStorage = GenericStorage<Box<dyn SignalSource>>;
pub type SignalStorageSync = GenericStorage<Box<dyn SignalSource + Send + Sync>>;

pub trait SignalSource: 'static {
    fn read(&self) -> &dyn Any;
    fn write(&mut self) -> &mut dyn Any;
    fn set(&mut self, new: Box<dyn Any>);
    fn as_any(&mut self) -> &mut dyn Any;
}

pub struct ReadOnly;
pub struct Writable;
pub struct Untracked;

pub trait SupportsWrites {}
impl SupportsWrites for Writable {}
impl SupportsWrites for Untracked {}

// Just a normal value with nothing special
// No subscriptions, etc
// Basically just a CopyValue
pub struct TrackedSource<T>(pub T);
impl<T: 'static> SignalSource for TrackedSource<T> {
    fn read(&self) -> &dyn Any {
        println!("Tracking read...");
        &self.0
    }
    fn write(&mut self) -> &mut dyn Any {
        println!("Tracking write...");
        &mut self.0
    }
    fn set(&mut self, new: Box<dyn Any>) {
        self.0 = *new.downcast().unwrap();
    }
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

/// A signal that's not tracked - modifications to this signal will not trigger updates.
/// Simplest signal imaginable!
pub struct UntrackedSource<T>(pub T);
impl<T: 'static> SignalSource for UntrackedSource<T> {
    fn read(&self) -> &dyn Any {
        &self.0
    }
    fn write(&mut self) -> &mut dyn Any {
        &mut self.0
    }
    fn set(&mut self, new: Box<dyn Any>) {
        self.0 = *new.downcast().unwrap();
    }
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

/// A signal that's not tracked - modifications to this signal will not trigger updates.
pub struct MemoSource<T> {
    computed: bool,
    value: T,
}
impl<T> MemoSource<T> {
    /// Lazily compute the value
    fn compute_chain(&mut self) {
        self.computed = true;
    }
}

impl<T: 'static> SignalSource for MemoSource<T> {
    fn read(&self) -> &dyn Any {
        &self.value
    }
    fn write(&mut self) -> &mut dyn Any {
        &mut self.value
    }
    fn set(&mut self, new: Box<dyn Any>) {
        self.value = *new.downcast().unwrap();
    }
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
