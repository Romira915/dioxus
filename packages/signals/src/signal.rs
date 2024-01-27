use std::{
    any::Any,
    cell::Cell,
    fmt::{Debug, Display},
    io::prelude::Write,
    mem::MaybeUninit,
    ops::{Add, Deref, DerefMut, Div, Mul, Sub},
    rc::Rc,
};

use dioxus_core::prelude::{has_context, provide_context, IntoAttributeValue, ScopeId};
use generational_box::{
    BoxMethods, GenerationalBox, MaybeSinkBox, Owner, Slot, SyncSlot, UnsyncSlot,
};

use crate::{
    current_owner, BoxSyncSignal, BoxUnsyncSignal, GlobalReadable, GlobalWritable, ReadOnly,
    Readable, ReadableVecExt, SignalSlot, Source, SupportsWrites, SyncSignalSlot, TrackedSource,
    UnsyncSignalSlot, Untracked, UntrackedSource, Writable, WritableMarker,
};

pub fn use_signal<T: 'static>(f: impl FnOnce() -> T) -> WriteSignal<T> {
    todo!()
}

pub fn use_signal_sync<T: 'static>(f: impl FnOnce() -> T) -> WriteSignal<T, SyncSignalSlot> {
    todo!()
}

pub struct Signal<T, S: SignalSlot = UnsyncSignalSlot, M = WritableMarker> {
    generational_box: GenerationalBox<S>,
    _marker: std::marker::PhantomData<(T, M, S)>,
}

// There's no way to create or modify a syncslot if the value itself is not sync.
// This lets us store `Box<dyn Source>`` in a syncslot while also guaranteeing that the value is sync
// Usually the Send/Sync is automatic, but technically box dyn source is two different types
// Note that this might be fragile if we change the innerworkings of what powers source
unsafe impl<T: Send + Sync, M> Send for Signal<T, SyncSlot<BoxUnsyncSignal>, M> {}
unsafe impl<T: Send + Sync, M> Sync for Signal<T, SyncSlot<BoxUnsyncSignal>, M> {}

pub type SyncSignal<T, S = SyncSignalSlot> = Signal<T, S, WritableMarker>;
pub type ReadOnlySignal<T, S = UnsyncSignalSlot> = Signal<T, S, ReadOnly>;
pub type WriteSignal<T, S = UnsyncSignalSlot> = Signal<T, S, WritableMarker>;
pub type UntrackedSignal<T, S = UnsyncSignalSlot> = Signal<T, S, WritableMarker>;
pub type CopyValue<T, S = UnsyncSignalSlot> = UntrackedSignal<T, S>;

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

impl<T: 'static, S: SignalSlot> CopyValue<T, S> {
    pub fn new_in_scope(value: T, scope: ScopeId) -> Self {
        let caller = std::panic::Location::caller();
        let src: Box<dyn Source> = Box::new(TrackedSource { value });
        Signal {
            generational_box: current_owner::<S>().insert(src),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: 'static, S: SignalSlot> GlobalMemo<T, S> {
    pub fn global_memo(value: fn() -> T) -> Self {
        let caller = std::panic::Location::caller();
        let src: Box<dyn Source> = Box::new(TrackedSource { value });
        Signal {
            generational_box: current_owner::<S>().insert(src),
            _marker: std::marker::PhantomData,
        }
    }
}
impl<T: 'static, S: SignalSlot> GlobalSignal<T, S> {
    pub const fn global(value: fn() -> T) -> Self {
        todo!()
        // let caller = std::panic::Location::caller();
        // let src: Box<dyn Source> = Box::new(TrackedSource { value });
        // Signal {
        //     generational_box: current_owner::<S>().insert(src),
        //     _marker: std::marker::PhantomData,
        // }
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

impl<T: 'static, S: SignalSlot, M> Readable<T> for Signal<T, S, M> {
    type Ref<R: ?Sized + 'static> = S::Ref<R>;

    fn map_ref<I: ?Sized, U: ?Sized, F: FnOnce(&I) -> &U>(
        ref_: Self::Ref<I>,
        f: F,
    ) -> Self::Ref<U> {
        S::map(ref_, f)
    }

    fn try_map_ref<I, U: ?Sized, F: FnOnce(&I) -> Option<&U>>(
        ref_: Self::Ref<I>,
        f: F,
    ) -> Option<Self::Ref<U>> {
        S::try_map(ref_, f)
    }

    fn read(&self) -> Self::Ref<T> {
        S::map(self.generational_box.read(), |f| {
            f.tracked_read();
            f.read().downcast_ref().unwrap()
        })
    }

    fn peek(&self) -> Self::Ref<T> {
        S::map(self.generational_box.read(), |f| {
            f.read().downcast_ref().unwrap()
        })
    }
}

impl<T: 'static, S: SignalSlot, M: SupportsWrites> Writable<T> for Signal<T, S, M> {
    type Mut<R: ?Sized + 'static> = S::Mut<R>;

    fn map_mut<I, U: ?Sized, F: FnOnce(&mut I) -> &mut U>(
        ref_: Self::Mut<I>,
        f: F,
    ) -> Self::Mut<U> {
        S::map_mut(ref_, f)
    }

    fn try_map_mut<I, U: ?Sized, F: FnOnce(&mut I) -> Option<&mut U>>(
        ref_: Self::Mut<I>,
        f: F,
    ) -> Option<Self::Mut<U>> {
        S::try_map_mut(ref_, f)
    }

    fn try_write(&self) -> Result<Self::Mut<T>, generational_box::error::BorrowMutError> {
        self.generational_box.try_write().map(|mut_| {
            S::map_mut(mut_, |f| {
                f.tracked_write();
                f.write().downcast_mut().unwrap()
            })
        })
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
        Signal::<T, S, WritableMarker>::new_maybe_sync(T::default())
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

impl<T, S: SignalSlot> Into<Signal<T, S, ReadOnly>> for Signal<T, S, WritableMarker> {
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
