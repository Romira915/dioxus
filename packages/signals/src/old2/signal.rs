use std::{
    any::Any,
    borrow::Borrow,
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    fmt::{Debug, Display},
    mem::MaybeUninit,
    ops::{Add, Deref, Div, Mul, Sub},
    rc::Rc,
    sync::Arc,
};

use dioxus_core::prelude::has_context;
use generational_box::{
    AnyStorage, GenerationalBox, GenerationalRef, GenerationalRefMut, GenericStorage,
    MemoryLocation, Owner, Storage,
};

use crate::{
    ReadOnly, SignalSource, SignalStorageSync, SupportsWrites, TrackedSource, UnsyncSignalStorage,
    Untracked, UntrackedSource, Writable,
};

pub fn use_signal<T>(f: impl FnOnce() -> T) -> Signal<T> {
    todo!()
}

pub fn use_signal_sync<T>(f: impl FnOnce() -> T) -> Signal<T, SignalStorageSync, Writable> {
    todo!()
}

/// A signal that implements Read/Write characteristics.
pub struct Signal<T, Store: Storage = UnsyncSignalStorage, M = Writable>
where
    T: ?Sized,
{
    inner: GenerationalBox<Box<dyn SignalSource>, Store>,
    _marker: std::marker::PhantomData<fn(T, M)>,
}

/// A base signal with no defaults
pub type BaseSignal<T, M> = Signal<T, M>;

/// A signal that only implements read characteristics.
pub type ReadOnlSignal<T> = Signal<T, UnsyncSignalStorage, ReadOnly>;

/// A signal that's not tracked - modifications to this signal will not trigger updates.
pub type UntrackedSignal<T> = Signal<T, UnsyncSignalStorage, Untracked>;

impl<T: 'static + Send + Sync> Signal<T, SignalStorageSync, Writable> {
    pub fn new_sync(value: T) -> Signal<T, SignalStorageSync, Writable> {
        let boxed = Box::new(TrackedSource(value)) as Box<dyn SignalSource + Send + Sync>;
        Signal {
            inner: sync_owner().insert(boxed),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: 'static> Signal<T, UnsyncSignalStorage> {
    /// Create a new signal with Write characteristics
    pub fn new(value: T) -> Self {
        Signal {
            inner: unsync_owner().insert(Box::new(TrackedSource(value)) as Box<dyn SignalSource>),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn into_read_only(&self) -> ReadOnlSignal<T> {
        ReadOnlSignal {
            inner: self.inner,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn untracked(value: T) -> Signal<T, UnsyncSignalStorage, Untracked> {
        Signal {
            inner: unsync_owner().insert(Box::new(UntrackedSource(value)) as Box<dyn SignalSource>),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: 'static, M: SupportsWrites> Signal<T, UnsyncSignalStorage, M> {
    pub fn write(&mut self) -> <GenericStorage<UnsyncSignalStorage> as AnyStorage>::Mut<T> {
        let inner = self.inner.write();

        UnsyncSignalStorage::map_mut(inner, |f| f.write().downcast_mut().unwrap())
    }

    pub fn set(&mut self, value: T) {
        *self.write() = value;
    }

    pub fn with_mut<O>(&mut self, f: impl FnOnce(&mut T) -> O) -> O {
        f(&mut *self.write())
    }
}

impl<T: 'static, S: SignalStorageA, M> Signal<T, S, M> {
    pub fn read(&self) -> <S as AnyStorage>::Ref<T> {
        let inner = self.inner.read();

        S::Source::map(inner, |f| f.read().downcast_ref().unwrap())
    }

    pub fn with<O>(&self, f: impl FnOnce(&T) -> O) -> O {
        f(&*self.read())
    }
}

impl<T: 'static> Signal<T, UnsyncSignalStorage, ReadOnly> {
    pub fn read_only(value: T) -> ReadOnlSignal<T> {
        ReadOnlSignal {
            inner: unsync_owner().insert(Box::new(TrackedSource(value)) as Box<dyn SignalSource>),
            _marker: std::marker::PhantomData,
        }
    }
}

// Degrade a writable signal to a readonly signal
impl<T> Into<ReadOnlSignal<T>> for Signal<T> {
    fn into(self) -> ReadOnlSignal<T> {
        ReadOnlSignal {
            inner: self.inner,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: ?Sized, M, S: SignalStorageA + 'static> Copy for Signal<T, S, M> {}
impl<T: ?Sized, M, S: SignalStorageA> Clone for Signal<T, S, M> {
    fn clone(&self) -> Self {
        Signal {
            inner: self.inner.clone(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: Display + 'static, S: SignalStorageA, M> Display for Signal<T, S, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let read = Signal::<T, S, M>::read(self);
        write!(f, "{}", read)
    }
}

impl<T, S: SignalStorageA, M> PartialEq for Signal<T, S, M> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.ptr_eq(&other.inner)
    }
}

impl<T: Debug + 'static, S: SignalStorageA, M> Debug for Signal<T, S, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let read = Signal::<T, S, M>::read(self);
        write!(f, "{:?}", read)
    }
}

impl<T: Add<Output = T> + Copy + 'static, S: SignalStorageA, M> std::ops::Add<T>
    for Signal<T, S, M>
{
    type Output = T;

    #[track_caller]
    fn add(self, rhs: T) -> Self::Output {
        self.with(|v| *v + rhs)
    }
}

impl<M: SupportsWrites, T: Add<Output = T> + Copy + 'static> std::ops::AddAssign<T>
    for Signal<T, UnsyncSignalStorage, M>
{
    #[track_caller]
    fn add_assign(&mut self, rhs: T) {
        self.with_mut(|v| *v = *v + rhs)
    }
}

impl<M: SupportsWrites, T: Sub<Output = T> + Copy + 'static> std::ops::SubAssign<T>
    for Signal<T, UnsyncSignalStorage, M>
{
    #[track_caller]
    fn sub_assign(&mut self, rhs: T) {
        self.with_mut(|v| *v = *v - rhs)
    }
}

impl<M: SupportsWrites, T: Sub<Output = T> + Copy + 'static> std::ops::Sub<T>
    for Signal<T, UnsyncSignalStorage, M>
{
    type Output = T;

    #[track_caller]
    fn sub(self, rhs: T) -> Self::Output {
        self.with(|v| *v - rhs)
    }
}

impl<M: SupportsWrites, T: Mul<Output = T> + Copy + 'static> std::ops::MulAssign<T>
    for Signal<T, UnsyncSignalStorage, M>
{
    #[track_caller]
    fn mul_assign(&mut self, rhs: T) {
        self.with_mut(|v| *v = *v * rhs)
    }
}

impl<M: SupportsWrites, T: Mul<Output = T> + Copy + 'static> std::ops::Mul<T>
    for Signal<T, UnsyncSignalStorage, M>
{
    type Output = T;

    #[track_caller]
    fn mul(self, rhs: T) -> Self::Output {
        self.with(|v| *v * rhs)
    }
}

impl<M: SupportsWrites, T: Div<Output = T> + Copy + 'static> std::ops::DivAssign<T>
    for Signal<T, UnsyncSignalStorage, M>
{
    #[track_caller]
    fn div_assign(&mut self, rhs: T) {
        self.with_mut(|v| *v = *v / rhs)
    }
}

impl<M: SupportsWrites, T: Div<Output = T> + Copy + 'static> std::ops::Div<T>
    for Signal<T, UnsyncSignalStorage, M>
{
    type Output = T;

    #[track_caller]
    fn div(self, rhs: T) -> Self::Output {
        self.with(|v| *v / rhs)
    }
}

/// Currently only limited to copy types, though could probably specialize for string/arc/rc
impl<T: Clone + 'static, S: SignalStorageA + 'static, M> Deref for Signal<T, S, M> {
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

fn sync_owner() -> Arc<Owner<SignalStorageSync>> {
    thread_local! {
        static DEFAULT_OWNER: Arc<Owner<SignalStorageSync>> = Arc::new(SignalStorageSync::owner());
    }

    match has_context() {
        Some(owner) => owner,
        None => DEFAULT_OWNER.with(|owner| owner.clone()),
    }
}

fn unsync_owner() -> Rc<Owner<UnsyncSignalStorage>> {
    thread_local! {
        static DEFAULT_OWNER: Rc<Owner<UnsyncSignalStorage>> = Rc::new(UnsyncSignalStorage::owner());
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
        d: ReadOnlSignal<i32>,
        e: ReadOnlSignal<String>,
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
        let owner = UnsyncSignalStorage::owner();

        let out = owner.insert(Box::new(TrackedSource(123_i32)));

        *out.write() = Box::new(TrackedSource(456)) as Box<dyn SignalSource>;

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
