mod references;
use error::{BorrowError, BorrowMutError, ValueDroppedError};
use references::*;

pub mod error;
// mod storage;
mod arena;
mod freelist;
mod maybesink;
mod slot;
mod sync;
mod unsync;

pub use arena::*;
pub use freelist::*;
pub use maybesink::*;
pub use slot::*;
pub use sync::*;
pub use unsync::*;

pub struct GenerationalBox<S: 'static> {
    slot: &'static S,

    #[cfg(any(debug_assertions, feature = "check_generation"))]
    generation: u32,

    #[cfg(any(debug_assertions, feature = "debug_ownership"))]
    created_at: &'static std::panic::Location<'static>,
}

impl<S: Slot> GenerationalBox<S> {
    /// Get the id of the generational box.
    pub fn id(&self) -> GenerationalBoxId {
        GenerationalBoxId {
            data_ptr: self.slot.data_ptr(),
            #[cfg(any(debug_assertions, feature = "check_generation"))]
            generation: self.generation,
        }
    }

    #[inline(always)]
    pub(crate) fn validate(&self) -> bool {
        #[cfg(any(debug_assertions, feature = "check_generation"))]
        {
            self.slot.generation() == self.generation
        }
        #[cfg(not(any(debug_assertions, feature = "check_generation")))]
        {
            true
        }
    }
    /// Try to read the value. Returns None if the value is no longer valid.
    #[track_caller]
    pub fn try_read(&self) -> Result<S::Ref<S::Item>, BorrowError> {
        if !self.validate() {
            return Err(BorrowError::Dropped(ValueDroppedError {
                #[cfg(any(debug_assertions, feature = "debug_borrows"))]
                created_at: self.created_at,
            }));
        }
        let result = self.slot.try_read(
            #[cfg(any(debug_assertions, feature = "debug_ownership"))]
            GenerationalRefBorrowInfo {
                borrowed_at: std::panic::Location::caller(),
                borrowed_from: &self.slot.borrowed(),
                created_at: self.created_at,
            },
        );

        if result.is_ok() {
            #[cfg(any(debug_assertions, feature = "debug_ownership"))]
            self.slot
                .borrowed()
                .borrowed_at
                .write()
                .push(std::panic::Location::caller());
        }

        result
    }

    /// Read the value. Panics if the value is no longer valid.
    #[track_caller]
    pub fn read(&self) -> S::Ref<S::Item> {
        self.try_read().unwrap()
    }

    /// Try to write the value. Returns None if the value is no longer valid.
    #[track_caller]
    pub fn try_write(&self) -> Result<S::Mut<S::Item>, BorrowMutError> {
        if !self.validate() {
            return Err(BorrowMutError::Dropped(ValueDroppedError {
                #[cfg(any(debug_assertions, feature = "debug_borrows"))]
                created_at: self.created_at,
            }));
        }
        let result = self.slot.try_write(
            #[cfg(any(debug_assertions, feature = "debug_ownership"))]
            GenerationalRefMutBorrowInfo {
                borrowed_from: &self.slot.borrowed(),
                created_at: self.created_at,
            },
        );

        if result.is_ok() {
            #[cfg(any(debug_assertions, feature = "debug_ownership"))]
            {
                *self.slot.borrowed().borrowed_mut_at.write() =
                    Some(std::panic::Location::caller());
            }
        }

        result
    }

    /// Write the value. Panics if the value is no longer valid.
    #[track_caller]
    pub fn write(&self) -> S::Mut<S::Item> {
        self.try_write().unwrap()
    }

    /// Set the value. Panics if the value is no longer valid.
    pub fn set(&self, value: S::Item) -> Option<S::Item> {
        if !self.validate() {
            return None;
        }

        self.slot.set(Some(value))
    }

    /// Returns true if the pointer is equal to the other pointer.
    pub fn ptr_eq(&self, other: &Self) -> bool {
        #[cfg(any(debug_assertions, feature = "check_generation"))]
        {
            self.slot.data_ptr() == other.slot.data_ptr() && self.generation == other.generation
        }
        #[cfg(not(any(debug_assertions, feature = "check_generation")))]
        {
            self.data.data_ptr() == other.data.data_ptr()
        }
    }
}

/// The type erased id of a generational box.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct GenerationalBoxId {
    data_ptr: usize,
    #[cfg(any(debug_assertions, feature = "check_generation"))]
    generation: u32,
}

impl<S: 'static> Copy for GenerationalBox<S> {}

impl<T: Slot> PartialEq for GenerationalBox<T> {
    fn eq(&self, other: &Self) -> bool {
        self.slot.data_ptr() == other.slot.data_ptr()
    }
}

impl<S> Clone for GenerationalBox<S> {
    fn clone(&self) -> Self {
        *self
    }
}
