use std::{cell::RefCell, sync::Mutex};

use crate::{sync::SyncSlot, unsync::UnsyncSlot, GenerationalBox, Slot};

pub trait Freelist: Default {
    type List;
    type Item;
    type Slot: Slot<Item = Self::Item>;
    fn alloc(
        &self,
        #[cfg(any(debug_assertions, feature = "debug_ownership"))]
        created_at: &'static std::panic::Location<'static>,
    ) -> GenerationalBox<Self::Slot>;
    fn free(&self, entry: GenerationalBox<Self::Slot>) -> Option<Self::Item>;
}

pub struct UnsyncFreelist<T: 'static> {
    list: RefCell<Vec<&'static UnsyncSlot<T>>>,
}

impl<T> Freelist for UnsyncFreelist<T> {
    type List = RefCell<Vec<&'static UnsyncSlot<T>>>;
    type Slot = UnsyncSlot<T>;
    type Item = T;

    fn alloc(
        &self,
        #[cfg(any(debug_assertions, feature = "debug_ownership"))]
        created_at: &'static std::panic::Location<'static>,
    ) -> GenerationalBox<Self::Slot> {
        let slot = match self.list.borrow_mut().pop() {
            Some(slot) => slot,
            None => Box::leak(Box::new(UnsyncSlot::default())),
        };

        GenerationalBox {
            slot,
            #[cfg(any(debug_assertions, feature = "check_generation"))]
            generation: slot.generation(),
            #[cfg(any(debug_assertions, feature = "debug_ownership"))]
            created_at,
        }
    }

    fn free(&self, entry: GenerationalBox<Self::Slot>) -> Option<T> {
        self.list.borrow_mut().push(entry.slot);
        entry.slot.increment_generation();
        entry.slot.set(None)
    }
}

impl<T> Default for UnsyncFreelist<T> {
    fn default() -> Self {
        Self {
            list: RefCell::new(Vec::new()),
        }
    }
}

pub struct SyncFreeList<T: 'static> {
    list: Mutex<Vec<&'static SyncSlot<T>>>,
}

// Let the sync-free list *technically* bye send/sync for unsync values
// We prevent the creation of sync slots for unsync values, so this is safe
unsafe impl<T> Send for SyncFreeList<T> {}
unsafe impl<T> Sync for SyncFreeList<T> {}

impl<T> Freelist for SyncFreeList<T> {
    type List = Mutex<Vec<&'static SyncSlot<T>>>;
    type Slot = SyncSlot<T>;
    type Item = T;

    fn alloc(
        &self,
        #[cfg(any(debug_assertions, feature = "debug_ownership"))]
        created_at: &'static std::panic::Location<'static>,
    ) -> GenerationalBox<Self::Slot> {
        let slot = match self.list.lock().unwrap().pop() {
            Some(slot) => slot,
            None => Box::leak(Box::new(SyncSlot::default())),
        };

        GenerationalBox {
            slot,
            #[cfg(any(debug_assertions, feature = "check_generation"))]
            generation: slot.generation(),
            #[cfg(any(debug_assertions, feature = "debug_ownership"))]
            created_at,
        }
    }

    fn free(&self, entry: GenerationalBox<Self::Slot>) -> Option<T> {
        self.list.lock().unwrap().push(entry.slot);
        entry.slot.increment_generation();
        entry.slot.set(None)
    }
}

impl<T> Default for SyncFreeList<T> {
    fn default() -> Self {
        Self {
            list: Mutex::new(Vec::new()),
        }
    }
}
