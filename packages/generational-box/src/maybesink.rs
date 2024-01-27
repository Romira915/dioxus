use crate::{
    error::{BorrowError, BorrowMutError},
    GenerationalBox, Slot, SyncSlot, UnsyncSlot,
};

/// Expose a generational box's methods regardless of whether it is sync or not.
pub struct MaybeSinkBox<S: Slot> {
    inner: GenerationalBox<S>,
}

impl<T: 'static + Send + Sync> MaybeSinkBox<SyncSlot<T>> {}

pub trait BoxMethods {
    type S: Slot;

    fn try_read(&self) -> Result<<Self::S as Slot>::Ref<<Self::S as Slot>::Item>, BorrowError>;

    fn try_write(&self) -> Result<<Self::S as Slot>::Mut<<Self::S as Slot>::Item>, BorrowMutError>;

    fn set(&self, value: <Self::S as Slot>::Item) -> Option<<Self::S as Slot>::Item>;

    fn read(&self) -> <Self::S as Slot>::Ref<<Self::S as Slot>::Item> {
        self.try_read().unwrap()
    }

    fn write(&self) -> <Self::S as Slot>::Mut<<Self::S as Slot>::Item> {
        self.try_write().unwrap()
    }
}

impl<T: 'static + Send + Sync> BoxMethods for MaybeSinkBox<SyncSlot<T>> {
    type S = SyncSlot<T>;

    fn try_read(&self) -> Result<<Self::S as Slot>::Ref<<Self::S as Slot>::Item>, BorrowError> {
        self.inner.try_read()
    }

    fn try_write(&self) -> Result<<Self::S as Slot>::Mut<<Self::S as Slot>::Item>, BorrowMutError> {
        self.inner.try_write()
    }

    fn set(&self, value: <Self::S as Slot>::Item) -> Option<<Self::S as Slot>::Item> {
        self.inner.set(value)
    }
}

impl<T: 'static> BoxMethods for MaybeSinkBox<UnsyncSlot<T>> {
    type S = UnsyncSlot<T>;

    fn try_read(&self) -> Result<<Self::S as Slot>::Ref<<Self::S as Slot>::Item>, BorrowError> {
        self.inner.try_read()
    }

    fn try_write(&self) -> Result<<Self::S as Slot>::Mut<<Self::S as Slot>::Item>, BorrowMutError> {
        self.inner.try_write()
    }

    fn set(&self, value: <Self::S as Slot>::Item) -> Option<<Self::S as Slot>::Item> {
        self.inner.set(value)
    }
}
