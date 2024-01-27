use crate::freelist::*;
use crate::{GenerationalBox, Slot};
use std::marker::PhantomData;

pub type UnsyncArena<T> = Arena<T, UnsyncFreelist<T>>;
pub type SyncArena<T> = Arena<T, SyncFreeList<T>>;

pub struct Arena<T, F> {
    list: F,
    _p: PhantomData<T>,
}

impl Arena<(), ()> {
    pub fn new<T>() -> Arena<T, UnsyncFreelist<T>> {
        Arena {
            list: UnsyncFreelist::default(),
            _p: PhantomData,
        }
    }

    pub fn new_sync<T>() -> Arena<T, SyncFreeList<T>> {
        Arena {
            list: SyncFreeList::default(),
            _p: PhantomData,
        }
    }
}

impl<T: 'static, F: Freelist<T>> Arena<T, F> {
    /// Insert a new value into this arena
    pub fn insert(&self, item: T) -> GenerationalBox<T, F::Slot> {
        let slot = self.list.alloc();
        slot.slot.set(Some(item));
        slot
    }

    pub fn free(&self, entry: GenerationalBox<T, F::Slot>) -> Option<T> {
        self.list.free(entry)
    }

    pub fn owner<'a>(&'a self) -> Owner<'a, T, F> {
        Owner {
            arena: self,
            owned: Default::default(),
        }
    }
}

pub struct Owner<'a, T: 'static, F: Freelist<T> + 'static> {
    arena: &'a Arena<T, F>,
    owned: Vec<GenerationalBox<T, F::Slot>>,
}

impl<'a, T: 'static, F: Freelist<T>> Owner<'a, T, F> {
    pub fn insert(&mut self, item: T) -> GenerationalBox<T, F::Slot> {
        let item = self.arena.insert(item);
        self.owned.push(item);
        item
    }
}

impl<'a, T: 'static, F: Freelist<T>> Drop for Owner<'a, T, F> {
    fn drop(&mut self) {
        for item in self.owned.drain(..) {
            self.arena.free(item);
        }
    }
}
