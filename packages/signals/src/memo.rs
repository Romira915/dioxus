use crate::callback::use_callback;
use crate::signal::use_signal;
use crate::{Dependency, ReadOnlySignal, Signal, SignalSlot};
use dioxus_core::use_hook;

/// Creates a new unsync Selector. The selector will be run immediately and whenever any signal it reads changes.
///
/// Selectors can be used to efficiently compute derived data from signals.
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_signals::*;
///
/// fn App() -> Element {
///     let mut count = use_signal(|| 0);
///     let double = use_memo(move || count * 2);
///     count += 1;
///     assert_eq!(double.value(), count * 2);
///
///     rsx! { "{double}" }
/// }
/// ```
#[track_caller]
pub fn use_memo<R: PartialEq>(f: impl FnMut() -> R + 'static) -> Memo<R> {
    todo!()
}

pub struct Memo<T> {
    pub inner: ReadOnlySignal<T>,
}

impl<T: 'static> PartialEq for Memo<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<T> Copy for Memo<T> {}
impl<T> Clone for Memo<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> std::ops::Deref for Memo<T> {
    type Target = ReadOnlySignal<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> Into<ReadOnlySignal<T>> for Memo<T> {
    fn into(self) -> ReadOnlySignal<T> {
        self.inner
    }
}

/// Creates a new Selector that may be sync. The selector will be run immediately and whenever any signal it reads changes.
///
/// Selectors can be used to efficiently compute derived data from signals.
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_signals::*;
///
/// fn App(cx: Scope) -> Element {
///     let mut count = use_signal(cx, || 0);
///     let double = use_memo(cx, move || count * 2);
///     count += 1;
///     assert_eq!(double.value(), count * 2);
///
///     render! { "{double}" }
/// }
/// ```
#[track_caller]
pub fn use_maybe_sync_memo<R: PartialEq + 'static, S: SignalSlot>(
    memo: impl FnMut() -> R + 'static,
) -> ReadOnlySignal<R, S> {
    let mut callback = use_callback(memo);
    use_hook(|| Signal::maybe_sync_memo(move || callback.call()))
}

/// Creates a new unsync Selector with some local dependencies. The selector will be run immediately and whenever any signal it reads or any dependencies it tracks changes
///
/// Selectors can be used to efficiently compute derived data from signals.
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_signals::*;
///
/// fn App(cx: Scope) -> Element {
///     let mut local_state = use_state(cx, || 0);
///     let double = use_memo_with_dependencies(cx, (local_state.get(),), move |(local_state,)| local_state * 2);
///     local_state.set(1);
///
///     render! { "{double}" }
/// }
/// ```
#[track_caller]
pub fn use_memo_with_dependencies<R: PartialEq + 'static, D: Dependency>(
    dependencies: D,
    f: impl FnMut(D::Out) -> R + 'static,
) -> ReadOnlySignal<R>
where
    D::Out: 'static,
{
    use_maybe_sync_selector_with_dependencies(dependencies, f)
}

/// Creates a new Selector that may be sync with some local dependencies. The selector will be run immediately and whenever any signal it reads or any dependencies it tracks changes
///
/// Selectors can be used to efficiently compute derived data from signals.
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_signals::*;
///
/// fn App(cx: Scope) -> Element {
///     let mut local_state = use_state(cx, || 0);
///     let double = use_memo_with_dependencies(cx, (local_state.get(),), move |(local_state,)| local_state * 2);
///     local_state.set(1);
///
///     render! { "{double}" }
/// }
/// ```
#[track_caller]
pub fn use_maybe_sync_selector_with_dependencies<
    R: PartialEq + 'static,
    D: Dependency,
    S: SignalSlot,
>(
    dependencies: D,
    mut f: impl FnMut(D::Out) -> R + 'static,
) -> ReadOnlySignal<R, S>
where
    D::Out: 'static,
{
    let mut dependencies_signal = use_signal(|| dependencies.out());
    let selector = use_hook(|| {
        Signal::maybe_sync_memo(move || {
            let deref = &*dependencies_signal.read();
            f(deref.clone())
        })
    });
    let changed = { dependencies.changed(&*dependencies_signal.read()) };
    if changed {
        dependencies_signal.set(dependencies.out());
    }
    selector
}
