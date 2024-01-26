use std::{mem::MaybeUninit, ops::Deref};

use generational_box::{GenerationalBox, Storage, UnsyncStorage};

use crate::{ReadOnly, Source, SupportsWrites, Writable};

pub type ReadOnlySignal<T, Store = UnsyncStorage> = Signal<T, Store, ReadOnly>;

pub struct Signal<T, Store = UnsyncStorage, Src = Writable>
where
    T: 'static,               // The value
    Src: Source<T>,           // A type that lets us read but injects its own middleware
    Store: Storage<Src::Src>, // The actual backing of the middleware item
{
    inner: GenerationalBox<Src::Src, Store>,
}

impl<T, Store, Src> Signal<T, Store, Src>
where
    T: 'static,
    Src: Source<T>,
    Store: Storage<Src::Src>,
{
    pub fn read(&self) -> Store::Ref<T> {
        Src::read(&self.inner)
    }
}

impl<T, Store, Src> Signal<T, Store, Src>
where
    T: 'static,
    Src: Source<T> + SupportsWrites, // We can only write to signals that support writes
    Store: Storage<Src::Src>,
{
    pub fn write(&self) -> Store::Mut<T> {
        Src::write(&self.inner)
    }

    pub fn with_mut(&self, f: impl FnOnce(&mut T)) {
        f(&mut *self.write())
    }
}

/// Currently only limited to copy types, though could probably specialize for string/arc/rc
impl<T, Store, Src> Deref for Signal<T, Store, Src>
where
    T: 'static + Clone,
    Src: Source<T>,
    Store: Storage<Src::Src>,
{
    type Target = dyn Fn() -> T;

    fn deref(&self) -> &Self::Target {
        // https://github.com/dtolnay/case-studies/tree/master/callable-types

        // First we create a closure that captures something with the Same in memory layout as Self (MaybeUninit<Self>).
        let uninit_callable = MaybeUninit::<Self>::uninit();
        // Then move that value into the closure. We assume that the closure now has a in memory layout of Self.
        let uninit_closure = move || Self::read(unsafe { &*uninit_callable.as_ptr() }).clone();

        // Check that the size of the closure is the same as the size of Self in case the compiler changed the layout of the closure.
        let size_of_closure = std::mem::size_of_val(&uninit_closure);
        assert_eq!(size_of_closure, std::mem::size_of::<Self>());

        // Then cast the lifetime of the closure to the lifetime of &self.
        fn cast_lifetime<'a, T>(_a: &T, b: &'a T) -> &'a T {
            b
        }
        let reference_to_closure = cast_lifetime(
            {
                // The real closure that we will never use.
                &uninit_closure
            },
            // We transmute self into a reference to the closure. This is safe because we know that the closure has the same memory layout as Self so &Closure == &Self.
            unsafe { std::mem::transmute(self) },
        );

        // Cast the closure to a trait object.
        reference_to_closure as &Self::Target
    }
}

impl<T: 'static, S: Storage<T>> Into<Signal<T, S, ReadOnly>> for Signal<T, S, Writable> {
    fn into(self) -> Signal<T, S, ReadOnly> {
        Signal { inner: self.inner }
    }
}

impl<T, Store, Src> Clone for Signal<T, Store, Src>
where
    T: 'static,
    Src: Source<T>,
    Store: Storage<Src::Src>,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T, Store, Src> Copy for Signal<T, Store, Src>
where
    T: 'static,
    Src: Source<T>,
    Store: Storage<Src::Src>,
{
}

// #[test]
fn it_works() {
    fn api(a: Signal<i32>, b: ReadOnlySignal<i32>) {
        a.write();
        b.read();
        let p = a();
        let q = b();
    }

    fn composite(a: Signal<i32>) {
        api(a, a.into());
    }
}
