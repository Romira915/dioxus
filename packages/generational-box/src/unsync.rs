use crate::{error, GenerationalRefMut, MemoryLocationBorrowInfo};
use crate::{GenerationalRef, Slot};
use std::cell::{Cell, Ref, RefCell, RefMut};

pub struct UnsyncSlot<T> {
    data: RefCell<Option<T>>,
    generation: Cell<u32>,
    borrow: MemoryLocationBorrowInfo,
}

impl<T> Default for UnsyncSlot<T> {
    fn default() -> Self {
        Self {
            data: Default::default(),
            generation: Default::default(),
            borrow: Default::default(),
        }
    }
}

impl<T: 'static> Slot for UnsyncSlot<T> {
    type Ref<R: ?Sized + 'static> = GenerationalRef<Ref<'static, R>>;
    type Mut<W: ?Sized + 'static> = GenerationalRefMut<RefMut<'static, W>>;
    type Item = T;

    fn try_read(
        &'static self,
        #[cfg(any(debug_assertions, feature = "debug_ownership"))]
        at: crate::GenerationalRefBorrowInfo,
    ) -> Result<Self::Ref<T>, error::BorrowError> {
        let borrow = self.data.try_borrow();

        #[cfg(any(debug_assertions, feature = "debug_ownership"))]
        let borrow = borrow.map_err(|_| at.borrowed_from.borrow_error())?;

        #[cfg(not(any(debug_assertions, feature = "debug_ownership")))]
        let borrow = borrow.map_err(|_| {
            error::BorrowError::AlreadyBorrowedMut(error::AlreadyBorrowedMutError {})
        })?;

        Ref::filter_map(borrow, |any| any.as_ref())
            .map_err(|_| {
                error::BorrowError::Dropped(error::ValueDroppedError {
                    #[cfg(any(debug_assertions, feature = "debug_ownership"))]
                    created_at: at.created_at,
                })
            })
            .map(|guard| {
                GenerationalRef::new(
                    guard,
                    #[cfg(any(debug_assertions, feature = "debug_ownership"))]
                    at,
                )
            })
    }

    fn try_write(
        &'static self,
        #[cfg(any(debug_assertions, feature = "debug_ownership"))]
        at: crate::GenerationalRefMutBorrowInfo,
    ) -> Result<Self::Mut<T>, error::BorrowMutError> {
        let borrow = self.data.try_borrow_mut();

        #[cfg(any(debug_assertions, feature = "debug_ownership"))]
        let borrow = borrow.map_err(|_| at.borrowed_from.borrow_mut_error())?;

        #[cfg(not(any(debug_assertions, feature = "debug_ownership")))]
        let borrow = borrow
            .map_err(|_| error::BorrowMutError::AlreadyBorrowed(error::AlreadyBorrowedError {}))?;

        RefMut::filter_map(borrow, |any| any.as_mut())
            .map_err(|_| {
                error::BorrowMutError::Dropped(error::ValueDroppedError {
                    #[cfg(any(debug_assertions, feature = "debug_ownership"))]
                    created_at: at.created_at,
                })
            })
            .map(|guard| {
                GenerationalRefMut::new(
                    guard,
                    #[cfg(any(debug_assertions, feature = "debug_ownership"))]
                    at,
                )
            })
    }

    fn try_map<I: ?Sized, U: ?Sized + 'static>(
        _self: Self::Ref<I>,
        f: impl FnOnce(&I) -> Option<&U>,
    ) -> Option<Self::Ref<U>> {
        let GenerationalRef {
            inner,
            #[cfg(any(debug_assertions, feature = "debug_borrows"))]
            borrow,
            ..
        } = _self;
        Ref::filter_map(inner, f).ok().map(|inner| GenerationalRef {
            inner,
            #[cfg(any(debug_assertions, feature = "debug_borrows"))]
            borrow,
        })
    }

    fn try_map_mut<I: ?Sized, U: ?Sized + 'static>(
        mut_ref: Self::Mut<I>,
        f: impl FnOnce(&mut I) -> Option<&mut U>,
    ) -> Option<Self::Mut<U>> {
        let GenerationalRefMut {
            inner,
            #[cfg(any(debug_assertions, feature = "debug_borrows"))]
            borrow,
            ..
        } = mut_ref;
        RefMut::filter_map(inner, f)
            .ok()
            .map(|inner| GenerationalRefMut {
                inner,
                #[cfg(any(debug_assertions, feature = "debug_borrows"))]
                borrow: crate::GenerationalRefMutBorrowInfo {
                    borrowed_from: borrow.borrowed_from,
                    created_at: borrow.created_at,
                },
            })
    }

    fn set(&'static self, value: Option<T>) -> Option<T> {
        self.data.replace(value)
    }

    fn generation(&self) -> u32 {
        self.generation.get()
    }

    fn increment_generation(&self) -> u32 {
        self.generation.set(self.generation.get() + 1);
        self.generation.get()
    }

    fn borrowed(&'static self) -> &'static MemoryLocationBorrowInfo {
        &self.borrow
    }

    fn data_ptr(&'static self) -> usize {
        self.data.as_ptr() as usize
    }
}
