use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};

use crate::{
    error::{self, ValueDroppedError},
    GenerationalRef, GenerationalRefMut, MemoryLocationBorrowInfo, Slot,
};

pub struct SyncSlot<T> {
    data: RwLock<Option<T>>,
    generation: std::sync::atomic::AtomicU32,
    borrowed: MemoryLocationBorrowInfo,
}

impl<T> Default for SyncSlot<T> {
    fn default() -> Self {
        Self {
            data: Default::default(),
            generation: Default::default(),
            borrowed: Default::default(),
        }
    }
}

impl<V: 'static> Slot<V> for SyncSlot<V> {
    type Ref<R: ?Sized + 'static> = GenerationalRef<MappedRwLockReadGuard<'static, R>>;
    type Mut<W: ?Sized + 'static> = GenerationalRefMut<MappedRwLockWriteGuard<'static, W>>;

    fn try_read(
        &'static self,
        #[cfg(any(debug_assertions, feature = "debug_ownership"))]
        at: crate::GenerationalRefBorrowInfo,
    ) -> Result<Self::Ref<V>, crate::error::BorrowError> {
        let read = self.data.try_read();

        #[cfg(any(debug_assertions, feature = "debug_ownership"))]
        let read = read.ok_or_else(|| at.borrowed_from.borrow_error())?;

        #[cfg(not(any(debug_assertions, feature = "debug_ownership")))]
        let read = read.ok_or_else(|| {
            error::BorrowError::AlreadyBorrowedMut(error::AlreadyBorrowedMutError {})
        })?;

        RwLockReadGuard::try_map(read, |any| any.as_ref())
            .map_err(|_| {
                error::BorrowError::Dropped(ValueDroppedError {
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
    ) -> Result<Self::Mut<V>, crate::error::BorrowMutError> {
        let write = self.data.try_write();

        #[cfg(any(debug_assertions, feature = "debug_ownership"))]
        let write = write.ok_or_else(|| at.borrowed_from.borrow_mut_error())?;

        #[cfg(not(any(debug_assertions, feature = "debug_ownership")))]
        let write = write.ok_or_else(|| {
            error::BorrowMutError::AlreadyBorrowed(error::AlreadyBorrowedError {})
        })?;

        RwLockWriteGuard::try_map(write, |any| any.as_mut())
            .map_err(|_| {
                error::BorrowMutError::Dropped(ValueDroppedError {
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
        ref_: Self::Ref<I>,
        f: impl FnOnce(&I) -> Option<&U>,
    ) -> Option<Self::Ref<U>> {
        let GenerationalRef {
            inner,
            #[cfg(any(debug_assertions, feature = "debug_borrows"))]
            borrow,
            ..
        } = ref_;
        MappedRwLockReadGuard::try_map(inner, f)
            .ok()
            .map(|inner| GenerationalRef {
                inner,
                #[cfg(any(debug_assertions, feature = "debug_borrows"))]
                borrow: crate::GenerationalRefBorrowInfo {
                    borrowed_at: borrow.borrowed_at,
                    borrowed_from: borrow.borrowed_from,
                    created_at: borrow.created_at,
                },
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
        MappedRwLockWriteGuard::try_map(inner, f)
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

    fn set(&'static self, value: Option<V>) -> Option<V> {
        let mut data = self.data.write();
        let old = data.take();
        *data = value;
        old
    }

    fn generation(&self) -> u32 {
        self.generation.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn increment_generation(&self) -> u32 {
        self.generation
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    fn borrowed(&'static self) -> &'static MemoryLocationBorrowInfo {
        &self.borrowed
    }

    fn data_ptr(&'static self) -> usize {
        self.data.data_ptr() as _
    }
}
