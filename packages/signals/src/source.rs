use generational_box::{GenerationalBox, Storage};

pub trait Source<T: 'static>: 'static {
    type Src: 'static;
    fn read<S: Storage<Self::Src>>(b: &GenerationalBox<Self::Src, S>) -> S::Ref<T>;
    fn write<S: Storage<Self::Src>>(b: &GenerationalBox<Self::Src, S>) -> S::Mut<T>;
}

pub trait SupportsWrites {}
impl SupportsWrites for Writable {}
impl SupportsWrites for Untracked {}

/// Read-only marker
pub struct ReadOnly;
impl<T: 'static> Source<T> for ReadOnly {
    type Src = T;
    fn read<S: Storage<Self::Src>>(b: &GenerationalBox<Self::Src, S>) -> S::Ref<T> {
        b.read()
    }
    fn write<S: Storage<Self::Src>>(b: &GenerationalBox<Self::Src, S>) -> S::Mut<T> {
        // in theory we shouldn't allow this, but a unchecked write might be a valid use case to expose
        b.write()
    }
}

pub struct Writable;
impl<T: 'static> Source<T> for Writable {
    type Src = T;
    fn read<S: Storage<Self::Src>>(b: &GenerationalBox<Self::Src, S>) -> S::Ref<T> {
        b.read()
    }
    fn write<S: Storage<Self::Src>>(b: &GenerationalBox<Self::Src, S>) -> S::Mut<T> {
        b.write()
    }
}

pub struct Untracked;
impl<T: 'static> Source<T> for Untracked {
    type Src = T;
    fn read<S: Storage<Self::Src>>(b: &GenerationalBox<Self::Src, S>) -> S::Ref<T> {
        b.read()
    }
    fn write<S: Storage<Self::Src>>(b: &GenerationalBox<Self::Src, S>) -> S::Mut<T> {
        b.write()
    }
}
