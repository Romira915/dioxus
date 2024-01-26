use std::{
    any::Any,
    borrow::Borrow,
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    fmt::{Debug, Display},
    mem::MaybeUninit,
    ops::Deref,
    rc::Rc,
};

use dioxus_core::prelude::has_context;
use generational_box::{
    AnyStorage, GenerationalBox, GenerationalRef, GenerationalRefMut, GenericStorage,
    MemoryLocation, Owner, Storage,
};

use crate::{
    ReadOnly, SigSource, SignalStorage, SupportsWrites, TrackedSource, Untracked, UntrackedSource,
    Writable,
};

/// A signal that implements Read/Write characteristics.
pub struct Signal<T: ?Sized, M = Writable, S: 'static = SignalStorage> {
    inner: GenerationalBox<Box<dyn SigSource>, S>,
    _marker: std::marker::PhantomData<fn(T, M)>,
}

pub type BaseSignal<T, M> = Signal<T, M>;

/// A signal that only implements read characteristics.
pub type ReadSignal<T> = Signal<T, ReadOnly, SignalStorage>;

/// A signal that's not tracked - modifications to this signal will not trigger updates.
pub type UntrackedSignal<T> = Signal<T, Untracked, SignalStorage>;

impl<T: 'static> Signal<T, Writable, SignalStorage> {
    /// Create a new signal with Write characteristics
    pub fn new(value: T) -> Self {
        Signal {
            inner: owner().insert(Box::new(TrackedSource(value)) as Box<dyn SigSource>),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn into_read_only(&self) -> ReadSignal<T> {
        ReadSignal {
            inner: self.inner,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn untracked(value: T) -> Signal<T, Untracked, SignalStorage> {
        Signal {
            inner: owner().insert(Box::new(UntrackedSource(value)) as Box<dyn SigSource>),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: 'static, M: SupportsWrites> Signal<T, M, SignalStorage> {
    pub fn write(&mut self) -> <SignalStorage as AnyStorage>::Mut<T> {
        let inner = self.inner.write();

        SignalStorage::map_mut(inner, |f| f.write().downcast_mut().unwrap())
    }
}

impl<T: 'static, R> Signal<T, R, SignalStorage> {
    pub fn read(&self) -> <SignalStorage as AnyStorage>::Ref<T> {
        let inner = self.inner.read();

        SignalStorage::map(inner, |f| f.read().downcast_ref().unwrap())
    }
}

impl<T: 'static> Signal<T, ReadOnly, SignalStorage> {
    pub fn read_only(value: T) -> ReadSignal<T> {
        ReadSignal {
            inner: owner().insert(Box::new(TrackedSource(value)) as Box<dyn SigSource>),
            _marker: std::marker::PhantomData,
        }
    }
}

// Degrade a writable signal to a readonly signal
impl<T> Into<ReadSignal<T>> for Signal<T> {
    fn into(self) -> ReadSignal<T> {
        ReadSignal {
            inner: self.inner,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: ?Sized, M, S: 'static> Copy for Signal<T, M, S> {}
impl<T: ?Sized, M, S> Clone for Signal<T, M, S> {
    fn clone(&self) -> Self {
        Signal {
            inner: self.inner.clone(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: Display + 'static, S> Display for Signal<T, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let read = Signal::<T, S>::read(self);
        write!(f, "{}", read)
    }
}

impl<T: Debug + 'static, S> Debug for Signal<T, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let read = Signal::<T, S>::read(self);
        write!(f, "{:?}", read)
    }
}

///
/// Currently only limited to copy types, though could probably specialize for string/arc/rc
impl<T: Clone + 'static, S: 'static> Deref for Signal<T, S> {
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

fn owner() -> Rc<Owner<SignalStorage>> {
    thread_local! {
        static DEFAULT_OWNER: Rc<Owner<SignalStorage>> = Rc::new(SignalStorage::owner());
    }

    match has_context() {
        Some(owner) => owner,
        None => DEFAULT_OWNER.with(|owner| owner.clone()),
    }
}

#[test]
fn new_owner() {
    fn my_component(
        a: Signal<i32>,
        b: Signal<HashMap<i32, String>>,
        c: Signal<Box<dyn Fn() -> i32>>,
        d: ReadSignal<i32>,
        e: ReadSignal<String>,
        f: UntrackedSignal<i32>,
        g: UntrackedSignal<i32>,
    ) {
        println!("a: {}", a);
        println!("b: {:?}", b);
        println!("c: {}", c.read()());
        println!("d: {}", d);
        println!("e: {}", e);
        println!("f: {}", f);
        println!("g: {}", g());
    }

    {
        let owner = SignalStorage::owner();

        let out = owner.insert(Box::new(TrackedSource(123_i32)));

        *out.write() = Box::new(TrackedSource(456)) as Box<dyn SigSource>;

        let mut signal: Signal<i32> = Signal::new(123);

        let val = signal.read();
        assert_eq!(*val, 123);
        drop(val);

        let mut val = signal.write();
        *val = 456;
        println!("val: {}", *val);
    }

    let a = Signal::new(123);
    let b = Signal::new(HashMap::new());
    let c = Signal::new(Box::new(move || a()) as _);
    let d = a.clone().into();
    let e = Signal::read_only("hello".to_string());
    let f = Signal::untracked(123);
    let g = Signal::untracked(a()); // there's no way to get a "untracked" variant of a regular signal from a signal - this needs to be fixed

    my_component(a, b, c, d, e, f, g);
}
