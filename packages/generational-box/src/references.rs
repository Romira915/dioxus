use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
};

/// A reference to a value in a generational box.
pub struct GenerationalRef<R> {
    pub(crate) inner: R,
    #[cfg(any(debug_assertions, feature = "debug_borrows"))]
    pub(crate) borrow: GenerationalRefBorrowInfo,
}

impl<T: ?Sized + 'static, R: Deref<Target = T>> GenerationalRef<R> {
    pub(crate) fn new(
        inner: R,
        #[cfg(any(debug_assertions, feature = "debug_borrows"))] borrow: GenerationalRefBorrowInfo,
    ) -> Self {
        Self {
            inner,
            #[cfg(any(debug_assertions, feature = "debug_borrows"))]
            borrow,
        }
    }
}

impl<T: ?Sized + Debug, R: Deref<Target = T>> Debug for GenerationalRef<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.deref().fmt(f)
    }
}

impl<T: ?Sized + Display, R: Deref<Target = T>> Display for GenerationalRef<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.deref().fmt(f)
    }
}

impl<T: ?Sized + 'static, R: Deref<Target = T>> Deref for GenerationalRef<R> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

#[cfg(any(debug_assertions, feature = "debug_borrows"))]
/// Information about a borrow.
pub struct GenerationalRefBorrowInfo {
    pub(crate) borrowed_at: &'static std::panic::Location<'static>,
    pub(crate) borrowed_from: &'static MemoryLocationBorrowInfo,
    pub(crate) created_at: &'static std::panic::Location<'static>,
}

#[cfg(any(debug_assertions, feature = "debug_borrows"))]
impl Drop for GenerationalRefBorrowInfo {
    fn drop(&mut self) {
        self.borrowed_from
            .borrowed_at
            .write()
            .retain(|location| !std::ptr::eq(*location, self.borrowed_at as *const _));
    }
}

/// A mutable reference to a value in a generational box.
pub struct GenerationalRefMut<W> {
    pub(crate) inner: W,
    #[cfg(any(debug_assertions, feature = "debug_borrows"))]
    pub(crate) borrow: GenerationalRefMutBorrowInfo,
}

impl<T: ?Sized + 'static, R: DerefMut<Target = T>> GenerationalRefMut<R> {
    pub(crate) fn new(
        inner: R,
        #[cfg(any(debug_assertions, feature = "debug_borrows"))]
        borrow: GenerationalRefMutBorrowInfo,
    ) -> Self {
        Self {
            inner,
            #[cfg(any(debug_assertions, feature = "debug_borrows"))]
            borrow,
        }
    }
}

impl<T: ?Sized + 'static, W: DerefMut<Target = T>> Deref for GenerationalRefMut<W> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<T: ?Sized + 'static, W: DerefMut<Target = T>> DerefMut for GenerationalRefMut<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.deref_mut()
    }
}

#[cfg(any(debug_assertions, feature = "debug_borrows"))]
/// Information about a mutable borrow.
pub struct GenerationalRefMutBorrowInfo {
    /// The location where the borrow occurred.
    pub(crate) borrowed_from: &'static MemoryLocationBorrowInfo,
    pub(crate) created_at: &'static std::panic::Location<'static>,
}

#[cfg(any(debug_assertions, feature = "debug_borrows"))]
impl Drop for GenerationalRefMutBorrowInfo {
    fn drop(&mut self) {
        self.borrowed_from.borrowed_mut_at.write().take();
    }
}

use crate::error::*;

#[cfg(any(debug_assertions, feature = "debug_borrows"))]
#[derive(Debug, Default)]
pub struct MemoryLocationBorrowInfo {
    pub(crate) borrowed_at: parking_lot::RwLock<Vec<&'static std::panic::Location<'static>>>,
    pub(crate) borrowed_mut_at: parking_lot::RwLock<Option<&'static std::panic::Location<'static>>>,
}

#[cfg(any(debug_assertions, feature = "debug_ownership"))]
impl MemoryLocationBorrowInfo {
    pub fn borrow_mut_error(&self) -> BorrowMutError {
        if let Some(borrowed_mut_at) = self.borrowed_mut_at.read().as_ref() {
            BorrowMutError::AlreadyBorrowedMut(crate::error::AlreadyBorrowedMutError {
                #[cfg(any(debug_assertions, feature = "debug_borrows"))]
                borrowed_mut_at,
            })
        } else {
            BorrowMutError::AlreadyBorrowed(crate::error::AlreadyBorrowedError {
                #[cfg(any(debug_assertions, feature = "debug_borrows"))]
                borrowed_at: self.borrowed_at.read().clone(),
            })
        }
    }

    pub fn borrow_error(&self) -> BorrowError {
        BorrowError::AlreadyBorrowedMut(crate::error::AlreadyBorrowedMutError {
            #[cfg(any(debug_assertions, feature = "debug_ownership"))]
            borrowed_mut_at: self.borrowed_mut_at.read().unwrap(),
        })
    }
}
