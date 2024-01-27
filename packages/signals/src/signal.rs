use std::{
    any::Any,
    cell::Cell,
    fmt::{Debug, Display},
    mem::MaybeUninit,
    ops::{Add, Deref, DerefMut, Div, Mul, Sub},
    rc::Rc,
};

use dioxus_core::prelude::{has_context, provide_context, IntoAttributeValue};
use generational_box::{
    BoxMethods, GenerationalBox, MaybeSinkBox, Owner, Slot, SyncSlot, UnsyncSlot,
};

use crate::{
    current_owner, BoxSyncSignal, BoxUnsyncSignal, ReadOnly, SignalSlot, Source, SupportsWrites,
    SyncSignalSlot, TrackedSource, UnsyncSignalSlot, Untracked, UntrackedSource, Writable,
};

pub fn use_signal<T: 'static>(f: impl FnOnce() -> T) -> WriteSignal<T> {
    todo!()
}

pub fn use_signal_sync<T: 'static>(f: impl FnOnce() -> T) -> WriteSignal<T, SyncSignalSlot> {
    todo!()
}

pub struct Signal<T, S: SignalSlot = UnsyncSignalSlot, M = Writable> {
    generational_box: GenerationalBox<S>,
    _marker: std::marker::PhantomData<(T, M, S)>,
}

// There's no way to create or modify a syncslot if the value itself is not sync.
// This lets us store `Box<dyn Source>`` in a syncslot while also guaranteeing that the value is sync
// Usually the Send/Sync is automatic, but technically box dyn source is two different types
// Note that this might be fragile if we change the innerworkings of what powers source
unsafe impl<T: Send + Sync, M> Send for Signal<T, SyncSlot<BoxUnsyncSignal>, M> {}
unsafe impl<T: Send + Sync, M> Sync for Signal<T, SyncSlot<BoxUnsyncSignal>, M> {}

pub type SyncSignal<T, S = SyncSignalSlot> = Signal<T, S, Writable>;
pub type ReadOnlySignal<T, S = UnsyncSignalSlot> = Signal<T, S, ReadOnly>;
pub type WriteSignal<T, S = UnsyncSignalSlot> = Signal<T, S, Writable>;
pub type UntrackedSignal<T, S = UnsyncSignalSlot> = Signal<T, S, Writable>;

impl<T: 'static> Signal<T> {
    pub fn new(value: T) -> Signal<T> {
        let caller = std::panic::Location::caller();
        let src: Box<dyn Source> = Box::new(TrackedSource { value });
        Signal {
            generational_box: current_owner::<UnsyncSignalSlot>().insert(src),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn untracked(value: T) -> UntrackedSignal<T> {
        let caller = std::panic::Location::caller();
        let src: Box<dyn Source> = Box::new(UntrackedSource { value });
        Signal {
            generational_box: current_owner::<UnsyncSignalSlot>().insert(src),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn read_only(value: T) -> ReadOnlySignal<T> {
        let caller = std::panic::Location::caller();
        let src: Box<dyn Source> = Box::new(TrackedSource { value });
        Signal {
            generational_box: current_owner::<UnsyncSignalSlot>().insert(src),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: 'static, S: SignalSlot> Signal<T, S> {
    pub fn new_maybe_sync<M>(value: T) -> Signal<T, S, M> {
        let caller = std::panic::Location::caller();
        let src: Box<dyn Source> = Box::new(TrackedSource { value });
        Signal {
            generational_box: current_owner::<S>().insert(src),
            _marker: std::marker::PhantomData,
        }
    }

    /// Creates a new Selector that may be Sync + Send. The selector will be run immediately and whenever any signal it reads changes.
    ///
    /// Selectors can be used to efficiently compute derived data from signals.
    #[track_caller]
    pub fn maybe_sync_memo(mut f: impl FnMut() -> T + 'static) -> ReadOnlySignal<T, S> {
        todo!("Implement memos")
        // let effect = Effect {
        //     source: current_scope_id().expect("in a virtual dom"),
        //     inner: CopyValue::invalid(),
        // };

        // {
        //     EFFECT_STACK.with(|stack| stack.effects.write().push(effect));
        // }
        // let mut state: Signal<T, S> = Signal::new_maybe_sync(f());
        // {
        //     EFFECT_STACK.with(|stack| stack.effects.write().pop());
        // }

        // let invalid_id = effect.id();
        // tracing::trace!("Creating effect: {:?}", invalid_id);
        // effect.inner.value.set(Box::new(EffectInner {
        //     callback: Box::new(move || {
        //         let value = f();
        //         let changed = {
        //             let old = state.inner.read();
        //             value != old.value
        //         };
        //         if changed {
        //             state.set(value)
        //         }
        //     }),
        //     id: invalid_id,
        // }));
        // {
        //     EFFECT_STACK.with(|stack| stack.effect_mapping.write().insert(invalid_id, effect));
        // }

        // ReadOnlySignal::new_maybe_sync(state)
    }
}

impl<T: 'static, S: SignalSlot, M> Signal<T, S, M> {
    /// Get the inner source type of the signal.
    pub fn source(&self) -> S::Ref<S::Item> {
        self.generational_box.read()
    }

    pub fn read(&self) -> S::Ref<T> {
        S::map(self.generational_box.read(), |f| {
            f.tracked_read();
            f.read().downcast_ref().unwrap()
        })
    }

    pub fn cloned(&self) -> T
    where
        T: Clone,
    {
        self.read().deref().clone()
    }

    pub fn read_untracked(&self) -> S::Ref<T> {
        S::map(self.generational_box.read(), |f| {
            f.read().downcast_ref().unwrap()
        })
    }

    pub fn peek(&self) -> S::Ref<T> {
        self.read_untracked()
    }

    pub fn with<O>(&self, f: impl FnOnce(&T) -> O) -> O {
        f(self.read().deref())
    }

    /// Run a function with a reference to the value. If the value has been dropped, this will panic.
    #[track_caller]
    pub fn with_peek<O>(&self, f: impl FnOnce(&T) -> O) -> O {
        f(&*self.peek())
    }

    /// Index into the inner value and return a reference to the result. If the value has been dropped or the index is invalid, this will panic.
    #[track_caller]
    pub fn index<I>(&self, index: I) -> S::Ref<T::Output>
    where
        T: std::ops::Index<I>,
    {
        S::map(self.read(), |v| v.index(index))
    }
}

impl<T: 'static, S: SignalSlot, M> Signal<Vec<T>, S, M> {
    /// Returns the length of the inner vector.
    #[track_caller]
    pub fn len(&self) -> usize {
        self.with(|v| v.len())
    }

    /// Returns true if the inner vector is empty.
    #[track_caller]
    pub fn is_empty(&self) -> bool {
        self.with(|v| v.is_empty())
    }

    /// Get the first element of the inner vector.
    #[track_caller]
    pub fn first(&self) -> Option<S::Ref<T>> {
        S::try_map(self.read(), |v| v.first())
    }

    /// Get the last element of the inner vector.
    #[track_caller]
    pub fn last(&self) -> Option<S::Ref<T>> {
        S::try_map(self.read(), |v| v.last())
    }

    /// Get the element at the given index of the inner vector.
    #[track_caller]
    pub fn get(&self, index: usize) -> Option<S::Ref<T>> {
        S::try_map(self.read(), |v| v.get(index))
    }

    /// Get an iterator over the values of the inner vector.
    #[track_caller]
    pub fn iter(&self) -> ReadableValueIterator<'_, T, Self>
    where
        Self: Sized,
    {
        ReadableValueIterator {
            index: 0,
            value: self,
            phantom: std::marker::PhantomData,
        }
    }
}

/// An iterator over the values of a `Readable<Vec<T>>`.
pub struct ReadableValueIterator<'a, T, R> {
    index: usize,
    value: &'a R,
    phantom: std::marker::PhantomData<T>,
}

impl<'a, T: 'static, S: SignalSlot, M> Iterator
    for ReadableValueIterator<'a, T, Signal<Vec<T>, S, M>>
{
    type Item = S::Ref<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        self.index += 1;
        self.value.get(index)
    }
}

impl<T: 'static, S: SignalSlot, M: SupportsWrites> Signal<T, S, M> {
    pub fn write(&self) -> S::Mut<T> {
        S::map_mut(self.generational_box.write(), |f| {
            f.tracked_write();
            f.write().downcast_mut().unwrap()
        })
    }

    pub fn write_untracked(&self) -> S::Mut<T> {
        S::map_mut(self.generational_box.write(), |f| {
            f.write().downcast_mut().unwrap()
        })
    }

    pub fn with_mut<O>(&self, f: impl FnOnce(&mut T) -> O) -> O {
        f(self.write().deref_mut())
    }

    pub fn set(&self, value: T) {
        // todo: might need to use a different setter in the event the value is not initalized
        *self.write() = value;
    }

    /// Invert the boolean value of the signal. This will trigger an update on all subscribers.
    #[track_caller]
    pub fn toggle(&mut self)
    where
        T: std::ops::Not<Output = T> + Clone,
    {
        self.set(!self.read().clone());
    }

    /// Index into the inner value and return a reference to the result.
    #[track_caller]
    pub fn index_mut<I>(&self, index: I) -> S::Mut<T::Output>
    where
        T: std::ops::IndexMut<I>,
    {
        S::map_mut(self.write(), |v| v.index_mut(index))
    }

    /// Takes the value out of the Signal, leaving a Default in its place.
    #[track_caller]
    pub fn take(&self) -> T
    where
        T: Default,
    {
        self.with_mut(|v| std::mem::take(v))
    }

    /// Replace the value in the Signal, returning the old value.
    #[track_caller]
    pub fn replace(&self, value: T) -> T {
        self.with_mut(|v| std::mem::replace(v, value))
    }
}

// ,ethods for options
impl<T: 'static, S: SignalSlot, M: SupportsWrites> Signal<Option<T>, S, M> {
    /// Gets the value out of the Option, or inserts the given value if the Option is empty.
    pub fn get_or_insert(&self, default: T) -> S::Mut<T> {
        self.get_or_insert_with(|| default)
    }

    /// Gets the value out of the Option, or inserts the value returned by the given function if the Option is empty.
    pub fn get_or_insert_with(&self, default: impl FnOnce() -> T) -> S::Mut<T> {
        let borrow = self.read();
        if borrow.is_none() {
            drop(borrow);
            self.with_mut(|v| *v = Some(default()));
            S::map_mut(self.write(), |v| v.as_mut().unwrap())
        } else {
            S::map_mut(self.write(), |v| v.as_mut().unwrap())
        }
    }

    /// Attempts to write the inner value of the Option.
    #[track_caller]
    pub fn as_mut(&self) -> Option<S::Mut<T>> {
        S::try_map_mut(self.write(), |v: &mut Option<T>| v.as_mut())
    }
}

// Methods for vec
// eventually we might want to back vec with a smarter storage type for precise tracking
impl<T: 'static, S: SignalSlot, M: SupportsWrites> Signal<Vec<T>, S, M> {
    /// Pushes a new value to the end of the vector.
    #[track_caller]
    pub fn push(&mut self, value: T) {
        self.with_mut(|v| v.push(value))
    }

    /// Pops the last value from the vector.
    #[track_caller]
    pub fn pop(&mut self) -> Option<T> {
        self.with_mut(|v| v.pop())
    }

    /// Inserts a new value at the given index.
    #[track_caller]
    pub fn insert(&mut self, index: usize, value: T) {
        self.with_mut(|v| v.insert(index, value))
    }

    /// Removes the value at the given index.
    #[track_caller]
    pub fn remove(&mut self, index: usize) -> T {
        self.with_mut(|v| v.remove(index))
    }

    /// Clears the vector, removing all values.
    #[track_caller]
    pub fn clear(&mut self) {
        self.with_mut(|v| v.clear())
    }

    /// Extends the vector with the given iterator.
    #[track_caller]
    pub fn extend(&mut self, iter: impl IntoIterator<Item = T>) {
        self.with_mut(|v| v.extend(iter))
    }

    /// Truncates the vector to the given length.
    #[track_caller]
    pub fn truncate(&mut self, len: usize) {
        self.with_mut(|v| v.truncate(len))
    }

    /// Swaps two values in the vector.
    #[track_caller]
    pub fn swap_remove(&mut self, index: usize) -> T {
        self.with_mut(|v| v.swap_remove(index))
    }

    /// Retains only the values that match the given predicate.
    #[track_caller]
    pub fn retain(&mut self, f: impl FnMut(&T) -> bool) {
        self.with_mut(|v| v.retain(f))
    }

    /// Splits the vector into two at the given index.
    #[track_caller]
    pub fn split_off(&mut self, at: usize) -> Vec<T> {
        self.with_mut(|v| v.split_off(at))
    }

    /// Try to mutably get an element from the vector.
    #[track_caller]
    pub fn get_mut(&self, index: usize) -> Option<S::Mut<T>> {
        S::try_map_mut(self.write(), |v: &mut Vec<T>| v.get_mut(index))
    }

    /// Gets an iterator over the values of the vector.
    #[track_caller]
    pub fn iter_mut(&self) -> WritableValueIterator<T, Self>
    where
        Self: Sized + Clone,
    {
        WritableValueIterator {
            index: 0,
            value: self.clone(),
            phantom: std::marker::PhantomData,
        }
    }
}

/// An iterator over the values of a `Writable<Vec<T>>`.
pub struct WritableValueIterator<T, R> {
    index: usize,
    value: R,
    phantom: std::marker::PhantomData<T>,
}

impl<T: 'static, S: SignalSlot, M: SupportsWrites> Iterator
    for WritableValueIterator<T, Signal<Vec<T>, S, M>>
{
    type Item = S::Mut<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        self.index += 1;
        self.value.get_mut(index)
    }
}

impl<T, S: SignalSlot, M> IntoAttributeValue for Signal<T, S, M>
where
    T: Clone + IntoAttributeValue + 'static,
{
    fn into_value(self) -> dioxus_core::AttributeValue {
        self.with(|f| f.clone().into_value())
    }
}

impl<T: 'static, S: SignalSlot, M> PartialEq for Signal<T, S, M> {
    fn eq(&self, other: &Self) -> bool {
        self.generational_box == other.generational_box
    }
}

impl<T, S: SignalSlot, M> Copy for Signal<T, S, M> {}
impl<T, S: SignalSlot, M> Clone for Signal<T, S, M> {
    fn clone(&self) -> Self {
        Self {
            generational_box: self.generational_box.clone(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: Default + 'static, S: SignalSlot> Default for Signal<T, S> {
    fn default() -> Self {
        Signal::<T, S, Writable>::new_maybe_sync(T::default())
    }
}
impl<T: Display + 'static, S: SignalSlot, M> Display for Signal<T, S, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.with(|v| v.fmt(f))
    }
}

impl<T: Debug + 'static, S: SignalSlot, M> Debug for Signal<T, S, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.with(|v| v.fmt(f))
    }
}

impl<T, S: SignalSlot> Into<Signal<T, S, ReadOnly>> for Signal<T, S, Writable> {
    fn into(self) -> Signal<T, S, ReadOnly> {
        Signal {
            generational_box: self.generational_box,
            _marker: std::marker::PhantomData,
        }
    }
}
impl<M: SupportsWrites, T: Add<Output = T> + Copy + 'static, S: SignalSlot> std::ops::Add<T>
    for Signal<T, S, M>
{
    type Output = T;

    #[track_caller]
    fn add(self, rhs: T) -> Self::Output {
        self.with(|v| *v + rhs)
    }
}

impl<M: SupportsWrites, T: Add<Output = T> + Copy + 'static, S: SignalSlot> std::ops::AddAssign<T>
    for Signal<T, S, M>
{
    #[track_caller]
    fn add_assign(&mut self, rhs: T) {
        self.with_mut(|v| *v = *v + rhs)
    }
}

impl<M: SupportsWrites, T: Sub<Output = T> + Copy + 'static, S: SignalSlot> std::ops::SubAssign<T>
    for Signal<T, S, M>
{
    #[track_caller]
    fn sub_assign(&mut self, rhs: T) {
        self.with_mut(|v| *v = *v - rhs)
    }
}

impl<M: SupportsWrites, T: Sub<Output = T> + Copy + 'static, S: SignalSlot> std::ops::Sub<T>
    for Signal<T, S, M>
{
    type Output = T;

    #[track_caller]
    fn sub(self, rhs: T) -> Self::Output {
        self.with(|v| *v - rhs)
    }
}

impl<M: SupportsWrites, T: Mul<Output = T> + Copy + 'static, S: SignalSlot> std::ops::MulAssign<T>
    for Signal<T, S, M>
{
    #[track_caller]
    fn mul_assign(&mut self, rhs: T) {
        self.with_mut(|v| *v = *v * rhs)
    }
}

impl<M: SupportsWrites, T: Mul<Output = T> + Copy + 'static, S: SignalSlot> std::ops::Mul<T>
    for Signal<T, S, M>
{
    type Output = T;

    #[track_caller]
    fn mul(self, rhs: T) -> Self::Output {
        self.with(|v| *v * rhs)
    }
}

impl<M: SupportsWrites, T: Div<Output = T> + Copy + 'static, S: SignalSlot> std::ops::DivAssign<T>
    for Signal<T, S, M>
{
    #[track_caller]
    fn div_assign(&mut self, rhs: T) {
        self.with_mut(|v| *v = *v / rhs)
    }
}

impl<M: SupportsWrites, T: Div<Output = T> + Copy + 'static, S: SignalSlot> std::ops::Div<T>
    for Signal<T, S, M>
{
    type Output = T;

    #[track_caller]
    fn div(self, rhs: T) -> Self::Output {
        self.with(|v| *v / rhs)
    }
}

/// Currently only limited to copy types, though could probably specialize for string/arc/rc
impl<T: 'static + Clone, S: SignalSlot, M: 'static> Deref for Signal<T, S, M> {
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
        pub fn cast_lifetime<'a, T>(_a: &T, b: &'a T) -> &'a T {
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
