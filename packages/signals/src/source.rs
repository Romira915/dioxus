use std::any::Any;

use generational_box::GenericStorage;

pub type SignalStorage = GenericStorage<Box<dyn SigSource>>;

pub trait SigSource: 'static {
    fn read(&self) -> &dyn Any;
    fn write(&mut self) -> &mut dyn Any;
}

pub struct ReadOnly;
pub struct Writable;
pub struct Untracked;

// Just a normal value with nothing special
// No subscriptions, etc
// Basically just a CopyValue
pub struct TrackedSource<T>(pub T);
impl<T: 'static> SigSource for TrackedSource<T> {
    fn read(&self) -> &dyn Any {
        println!("Tracking read...");
        &self.0
    }
    fn write(&mut self) -> &mut dyn Any {
        println!("Tracking write...");
        &mut self.0
    }
}

/// A signal that's not tracked - modifications to this signal will not trigger updates.
/// Simplest signal imaginable!
pub struct UntrackedSource<T>(pub T);
impl<T: 'static> SigSource for UntrackedSource<T> {
    fn read(&self) -> &dyn Any {
        &self.0
    }
    fn write(&mut self) -> &mut dyn Any {
        &mut self.0
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
impl<T: 'static> SigSource for MemoSource<T> {
    fn read(&self) -> &dyn Any {
        &self.value
    }
    fn write(&mut self) -> &mut dyn Any {
        &mut self.value
    }
}
