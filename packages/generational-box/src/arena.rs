use std::sync::Arc;

use crate::freelist::*;
use crate::{GenerationalBox, Slot};
use parking_lot::Mutex;

pub type UnsyncArena<T> = Arena<UnsyncFreelist<T>>;
pub type SyncArena<T> = Arena<SyncFreeList<T>>;

pub struct Arena<F> {
    list: F,
}

impl Arena<()> {
    pub fn new<T>() -> UnsyncArena<T> {
        Arena {
            list: UnsyncFreelist::default(),
        }
    }

    pub fn new_sync<T>() -> SyncArena<T> {
        Arena {
            list: SyncFreeList::default(),
        }
    }
}

impl<F: Freelist> Arena<F> {
    pub fn insert(&self, item: F::Item) -> GenerationalBox<F::Slot> {
        let location = std::panic::Location::caller();
        let slot = self.list.alloc(location);
        slot.slot.set(Some(item));
        slot
    }

    pub fn free(&self, entry: GenerationalBox<F::Slot>) -> Option<F::Item> {
        self.list.free(entry)
    }

    pub fn owner(self: &Arc<Self>) -> Owner<F> {
        Owner {
            arena: self.clone(),
            owned: Default::default(),
        }
    }
}

pub struct Owner<F: Freelist + 'static> {
    arena: Arc<Arena<F>>,
    owned: Mutex<Vec<GenerationalBox<F::Slot>>>,
}

impl<F: Freelist> Owner<F> {
    pub fn insert(&self, item: F::Item) -> GenerationalBox<F::Slot> {
        let item = self.arena.insert(item);
        self.owned.lock().push(item);
        item
    }
}

impl<F: Freelist> Drop for Owner<F> {
    fn drop(&mut self) {
        for item in self.owned.lock().drain(..) {
            self.arena.free(item);
        }
    }
}
