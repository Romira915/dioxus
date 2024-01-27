use std::{cell::RefCell, marker::PhantomData, sync::Mutex};

use crate::{sync::SyncSlot, unsync::UnsyncSlot, GenerationalBox, Slot};

pub trait Freelist: Default {
    type List;
    type Item;
    type Slot: Slot<Self::Item>;
    fn alloc(&self) -> GenerationalBox<Self::Item, Self::Slot>;
    fn free(&self, entry: GenerationalBox<Self::Item, Self::Slot>) -> Option<Self::Item>;
}

pub struct UnsyncFreelist<T: 'static> {
    list: RefCell<Vec<&'static UnsyncSlot<T>>>,
}

impl<T> Freelist for UnsyncFreelist<T> {
    type List = RefCell<Vec<&'static UnsyncSlot<T>>>;
    type Slot = UnsyncSlot<T>;
    type Item = T;

    fn alloc(&self) -> GenerationalBox<T, Self::Slot> {
        let slot = match self.list.borrow_mut().pop() {
            Some(slot) => slot,
            None => Box::leak(Box::new(UnsyncSlot::default())),
        };

        GenerationalBox {
            slot,
            #[cfg(any(debug_assertions, feature = "check_generation"))]
            generation: slot.generation(),
            #[cfg(any(debug_assertions, feature = "debug_ownership"))]
            created_at: std::panic::Location::caller(),
            _p: PhantomData,
        }
    }

    fn free(&self, entry: GenerationalBox<T, Self::Slot>) -> Option<T> {
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

impl<T> Freelist for SyncFreeList<T> {
    type List = Mutex<Vec<&'static SyncSlot<T>>>;
    type Slot = SyncSlot<T>;
    type Item = T;

    fn alloc(&self) -> GenerationalBox<T, Self::Slot> {
        let slot = match self.list.lock().unwrap().pop() {
            Some(slot) => slot,
            None => Box::leak(Box::new(SyncSlot::default())),
        };

        GenerationalBox {
            slot,
            #[cfg(any(debug_assertions, feature = "check_generation"))]
            generation: slot.generation(),
            #[cfg(any(debug_assertions, feature = "debug_ownership"))]
            created_at: std::panic::Location::caller(),
            _p: PhantomData,
        }
    }

    fn free(&self, entry: GenerationalBox<T, Self::Slot>) -> Option<T> {
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
