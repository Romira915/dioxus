use std::{
    any::Any,
    rc::Rc,
    sync::{Arc, OnceLock},
};

use dioxus_core::prelude::{has_context, provide_context};
use generational_box::{
    Arena, Freelist, GenerationalBox, Owner, Slot, SyncArena, SyncFreeList, SyncSlot, UnsyncArena,
    UnsyncFreelist, UnsyncSlot,
};

pub type BoxUnsyncSignal = Box<dyn Source>;
pub type BoxSyncSignal = Box<dyn Source + Send + Sync>;

pub type SyncBox = GenerationalBox<SyncSlot<BoxSyncSignal>>;
pub type UnsyncBox = GenerationalBox<UnsyncSlot<BoxUnsyncSignal>>;

pub trait SignalSlot: Slot<Item = BoxUnsyncSignal> {
    type AssociatedFreeList: Freelist<Item = Self::Item, Slot = Self>;
    fn owner() -> Rc<Owner<Self::AssociatedFreeList>>;
}

impl SignalSlot for UnsyncSignalSlot {
    type AssociatedFreeList = UnsyncFreelist<BoxUnsyncSignal>;
    fn owner() -> Rc<Owner<Self::AssociatedFreeList>> {
        UNSYNC_SIGNALS.with(|s| Rc::new(s.owner()))
    }
}

impl SignalSlot for SyncSignalSlot {
    type AssociatedFreeList = SyncFreeList<BoxUnsyncSignal>;
    fn owner() -> Rc<Owner<Self::AssociatedFreeList>> {
        Rc::new(
            SYNC_ARENA
                .get_or_init(|| Arc::new(Arena::new_sync()))
                .owner(),
        )
    }
}

pub type UnsyncSignalSlot = UnsyncSlot<BoxUnsyncSignal>;
pub type SyncSignalSlot = SyncSlot<BoxUnsyncSignal>;

pub fn current_owner<S: SignalSlot>() -> Rc<Owner<S::AssociatedFreeList>> {
    match has_context() {
        Some(rt) => rt,
        None => provide_context(S::owner()),
    }
}

thread_local! {
    static UNSYNC_SIGNALS: Arc<UnsyncArena<BoxUnsyncSignal>> = Arc::new(Arena::new());
}

static SYNC_ARENA: OnceLock<Arc<SyncArena<BoxUnsyncSignal>>> = OnceLock::new();

pub struct ReadOnly;
pub struct WritableMarker;
pub struct Untracked;

pub struct GlobalWritable;
pub struct GlobalReadable;

pub trait SupportsWrites {}
impl SupportsWrites for WritableMarker {}
impl SupportsWrites for Untracked {}
impl SupportsWrites for GlobalWritable {}

// Tracks reads/writes
pub struct TrackedSource<T: 'static> {
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

impl<T> Source for TrackedSource<T> {
    fn read(&self) -> &dyn Any {
        todo!()
    }

    fn write(&mut self) -> &mut dyn Any {
        todo!()
    }
}

pub struct UntrackedSource<T: 'static> {
    pub value: T,
}

impl<T> Source for UntrackedSource<T> {
    fn read(&self) -> &dyn Any {
        todo!()
    }

    fn write(&mut self) -> &mut dyn Any {
        todo!()
    }
}
