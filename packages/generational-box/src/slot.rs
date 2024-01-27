use crate::{
    error::{BorrowError, BorrowMutError},
    GenerationalRefBorrowInfo, GenerationalRefMutBorrowInfo, MemoryLocationBorrowInfo,
};

pub trait Slot<V>: 'static {
    type Ref<R: ?Sized + 'static>;
    type Mut<W: ?Sized + 'static>;

    /// Try to read the value. Returns None if the value is no longer valid.
    fn try_read(
        &'static self,
        #[cfg(any(debug_assertions, feature = "debug_ownership"))] at: GenerationalRefBorrowInfo,
    ) -> Result<Self::Ref<V>, BorrowError>;

    /// Try to write the value. Returns None if the value is no longer valid.
    fn try_write(
        &'static self,
        #[cfg(any(debug_assertions, feature = "debug_ownership"))] at: GenerationalRefMutBorrowInfo,
    ) -> Result<Self::Mut<V>, BorrowMutError>;

    /// Try to map the mutable ref.
    fn try_map_mut<T: ?Sized, U: ?Sized + 'static>(
        mut_ref: Self::Mut<T>,
        f: impl FnOnce(&mut T) -> Option<&mut U>,
    ) -> Option<Self::Mut<U>>;

    /// Map the mutable ref.
    fn map_mut<T: ?Sized, U: ?Sized + 'static>(
        mut_ref: Self::Mut<T>,
        f: impl FnOnce(&mut T) -> &mut U,
    ) -> Self::Mut<U> {
        Self::try_map_mut(mut_ref, |v| Some(f(v))).unwrap()
    }

    /// Try to map the ref.
    fn try_map<T: ?Sized, U: ?Sized + 'static>(
        ref_: Self::Ref<T>,
        f: impl FnOnce(&T) -> Option<&U>,
    ) -> Option<Self::Ref<U>>;

    /// Map the ref.
    fn map<T: ?Sized, U: ?Sized + 'static>(
        ref_: Self::Ref<T>,
        f: impl FnOnce(&T) -> &U,
    ) -> Self::Ref<U> {
        Self::try_map(ref_, |v| Some(f(v))).unwrap()
    }

    /// Set the value, returning the old value if it exists
    fn set(&'static self, value: Option<V>) -> Option<V>;

    /// Get the generation of the slot itself
    fn generation(&self) -> u32;

    /// Increment the generation of the slot itself
    fn increment_generation(&self) -> u32;

    /// Set the location where the value was borrowed from
    fn borrowed(&'static self) -> &'static MemoryLocationBorrowInfo;

    fn data_ptr(&'static self) -> usize;
}
