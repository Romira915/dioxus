use crate::{
    error,
    references::{GenerationalRef, GenerationalRefMut},
    AnyStorage, MemoryLocation, MemoryLocationInner, Storage,
};
use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    marker::PhantomData,
};

/// An arena for a given type T
///
/// V Is not guaranteed to be Send/Sync, so this is not Send/Sync compatible
pub struct GenericStorage<V>(RefCell<Option<V>>);

impl<T> Default for GenericStorage<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: 'static> Storage<T> for GenericStorage<T> {
    fn try_read(
        &'static self,
        #[cfg(any(debug_assertions, feature = "debug_ownership"))]
        at: crate::GenerationalRefBorrowInfo,
    ) -> Result<Self::Ref<T>, error::BorrowError> {
        let borrow = self.0.try_borrow();

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
        let borrow = self.0.try_borrow_mut();

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

    fn set(&self, value: T) {
        *self.0.borrow_mut() = Some(value);
    }
}

thread_local! {
    static GENERIC_RUNTIME: RefCell<HashMap<TypeId, Box<dyn Any>>> = RefCell::new(HashMap::new());
}

impl<V: 'static> AnyStorage for GenericStorage<V> {
    type Ref<R: ?Sized + 'static> = GenerationalRef<Ref<'static, R>>;
    type Mut<W: ?Sized + 'static> = GenerationalRefMut<RefMut<'static, W>>;

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

    fn data_ptr(&self) -> *const () {
        self.0.as_ptr() as *const ()
    }

    fn take(&self) -> bool {
        self.0.borrow_mut().take().is_some()
    }

    fn claim() -> MemoryLocation<Self> {
        GENERIC_RUNTIME.with(|runtime| {
            let mut rt = runtime.borrow_mut();

            let entry = rt.entry(TypeId::of::<V>()).or_insert_with(|| {
                let t: Vec<MemoryLocation<GenericStorage<V>>> = Vec::new();
                Box::new(t) as Box<dyn Any>
            });

            let vec = entry
                .downcast_mut::<Vec<MemoryLocation<GenericStorage<V>>>>()
                .unwrap();

            let p = if let Some(location) = vec.pop() {
                location
            } else {
                let data: &'static MemoryLocationInner<GenericStorage<V>> =
                    &*Box::leak(Box::new(MemoryLocationInner::<GenericStorage<V>> {
                        data: Self::default(),
                        #[cfg(any(debug_assertions, feature = "check_generation"))]
                        generation: 0.into(),
                        #[cfg(any(debug_assertions, feature = "debug_borrows"))]
                        borrow: Default::default(),
                    }));
                MemoryLocation(data)
            };

            p
        })
    }

    fn recycle(location: &MemoryLocation<Self>) {
        location.drop();

        GENERIC_RUNTIME.with(|runtime| {
            let mut rt = runtime.borrow_mut();

            let entry = rt.entry(TypeId::of::<V>()).or_insert_with(|| {
                let t: Vec<MemoryLocation<GenericStorage<V>>> = Vec::new();
                Box::new(t) as Box<dyn Any>
            });

            let vec = entry
                .downcast_mut::<Vec<MemoryLocation<GenericStorage<V>>>>()
                .unwrap();

            vec.push(*location);
        });
    }

    fn owner() -> crate::Owner<Self> {
        crate::Owner {
            owned: Default::default(),
            phantom: PhantomData,
        }
    }
}
