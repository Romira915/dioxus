use parking_lot::Mutex;

use crate::freelist::*;
use crate::{GenerationalBox, Slot};
use std::marker::PhantomData;

// pub type UnsyncArena<T> = Arena<T, UnsyncFreelist<T>>;
// pub type SyncArena<T> = Arena<T, SyncFreeList<T>>;

pub struct Arena<F> {
    list: F,
}

impl<F> Arena<F> {
    pub fn new<T>() -> Arena<UnsyncFreelist<T>> {
        Arena {
            list: UnsyncFreelist::default(),
        }
    }

    pub fn new_sync<T>() -> Arena<SyncFreeList<T>> {
        Arena {
            list: SyncFreeList::default(),
        }
    }
}

impl<F: Freelist> Arena<F> {
    fn insert(&self, item: F::Item) -> GenerationalBox<F::Item, F::Slot> {
        let slot = self.list.alloc();
        slot.slot.set(Some(item));
        slot
    }

    fn free(&self, entry: GenerationalBox<F::Item, F::Slot>) -> Option<F::Item> {
        self.list.free(entry)
    }

    pub fn owner<'a>(&'a self) -> Owner<'a, F> {
        Owner {
            arena: self,
            owned: Default::default(),
        }
    }
}

pub struct Owner<'a, F: Freelist + 'static> {
    arena: &'a Arena<F>,
    owned: Mutex<Vec<GenerationalBox<F::Item, F::Slot>>>,
}

impl<'a, F: Freelist> Owner<'a, F> {
    pub fn insert(&mut self, item: F::Item) -> GenerationalBox<F::Item, F::Slot> {
        let item = self.arena.insert(item);
        self.owned.lock().push(item);
        item
    }
}

impl<'a, F: Freelist> Drop for Owner<'a, F> {
    fn drop(&mut self) {
        for item in self.owned.lock().drain(..) {
            self.arena.free(item);
        }
    }
}
