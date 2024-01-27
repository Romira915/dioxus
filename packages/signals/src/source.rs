use std::{any::Any, rc::Rc, sync::OnceLock};

use dioxus_core::prelude::{has_context, provide_context};
use generational_box::{
    Arena, Freelist, GenerationalBox, Owner, SyncArena, UnsyncArena, UnsyncSlot,
};

pub type UnsyncSignal = Box<dyn Source>;
pub type SyncSignal = Box<dyn Source + Send + Sync>;
pub type SyncBox = GenerationalBox<SyncSignal, SyncArena<SyncSignal>>;
pub type UnsyncBox = GenerationalBox<UnsyncSignal, UnsyncArena<UnsyncSignal>>;

pub trait SourceInner {
    fn read(&self) -> &dyn Any;
    fn write(&mut self) -> &mut dyn Any;
    fn tracked_read(&self);
    fn tracked_write(&mut self);
}

type SignalOwner<S: Freelist<UnsyncSignal>> = Owner<'static, S, UnsyncSignal>;

pub fn current_owner<S>() -> Rc<SignalOwner<S>> {
    match has_context() {
        Some(rt) => rt,
        None => {
            todo!()
            // let owner = Rc::new(S::owner());
            // provide_context(owner)
        }
    }
}

thread_local! {
    static UNSYNC_SIGNALS: Rc<UnsyncArena<UnsyncSignal>> = Rc::new(UnsyncArena::new());
}

static SYNC_ARENA: OnceLock<SyncArena<SyncSignal>> = OnceLock::new();

pub struct ReadOnly;
pub struct Writable;
pub struct Untracked;

pub trait SupportsWrites {}
impl SupportsWrites for Writable {}
impl SupportsWrites for Untracked {}

// Tracks reads/writes
pub struct SignalSource<T: 'static> {
    pub value: T,
}

pub trait Source: 'static {
    /// Read the value
    fn read(&self) -> &dyn Any;

    // Write the value
    fn write(&mut self) -> &mut dyn Any;

    /// Mark this value as read
    fn tracked_read(&self) {}

    /// Mark this value as written
    fn tracked_write(&mut self) {}
}
