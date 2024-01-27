use crate::callback::{use_callback, UseCallback};
use dioxus_core::use_hook;

/// Create a new effect. The effect will be run immediately and whenever any signal it reads changes.
/// The signal will be owned by the current component and will be dropped when the component is dropped.
pub fn use_effect(callback: impl FnMut() + 'static) {
    // Make a callback that's always current, to prevent stale data
    let mut callback = use_callback(callback);

    // Create an effect that runs the callback
    // use_hook(|| Effect::new(move || callback.call()));
}
