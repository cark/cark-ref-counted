use std::{ops::Deref, pin::Pin};

pub trait SmartPointerFamily {
    type Pointer<T: ?Sized>: SmartPointer<T>;
    fn new<T>(value: T) -> Self::Pointer<T>;
}

pub trait SmartPointer<T: ?Sized>: Deref<Target = T> {
    type Mark: SmartPointerFamily<Pointer<T> = Self>;
}

//pub trait CloneSmartPointer<T: ?Sized + Clone>: SmartPointer<T> + Clone {}

// pub trait RefCountFamily {
//     type Pointer<T: ?Sized>: RefCounted<T, Mark = Self, WeakPointer = Self::WeakPointer<T>>;
//     type WeakPointer<T: ?Sized>: WeakPointer<T, StrongMark = Self, StrongPointer = Self::Pointer<T>>;
//     fn new<T>(value: T) -> Self::Pointer<T>;
// }

pub trait RefCounted<T: ?Sized>: SmartPointer<T, Mark = Self::StrongMark> + Clone {
    type StrongMark: SmartPointerFamily<Pointer<T> = Self>;
    type WeakPointer: WeakPointer<T, StrongMark = Self::Mark, StrongPointer = Self>;
    fn new<U>(value: U) -> <Self::Mark as SmartPointerFamily>::Pointer<U> {
        Self::Mark::new(value)
    }
    fn as_ptr(this: &Self) -> *const T;
    fn downgrade(this: &Self) -> Self::WeakPointer;
    fn make_mut(this: &mut Self) -> &mut T
    where
        T: Clone;
    /// # Safety
    /// see [std::rc::Rc::increment_strong_count]
    unsafe fn increment_strong_count(ptr: *const T);
    /// # Safety
    /// see [std::rc::Rc::decrement_strong_count]
    unsafe fn decrement_strong_count(ptr: *const T);
    fn into_raw(this: Self) -> *const T;
    /// # Safety
    /// see [std::rc::Rc::from_raw]
    unsafe fn from_raw(ptr: *const T) -> Self;
    fn strong_count(this: &Self) -> usize;
    fn weak_count(this: &Self) -> usize;
    fn get_mut(this: &mut Self) -> Option<&mut T>;
    fn new_cyclic<F>(data_fn: F) -> Self
    where
        F: FnOnce(&Self::WeakPointer) -> T,
        T: Sized;
    fn pin(value: T) -> Pin<Self>
    where
        T: Sized;
    fn try_unwrap(this: Self) -> Result<T, Self>
    where
        T: Sized;
}

pub trait WeakPointer<T: ?Sized>: Clone {
    type StrongMark: SmartPointerFamily<Pointer<T> = Self::StrongPointer>;
    type StrongPointer: RefCounted<T, StrongMark = Self::StrongMark, WeakPointer = Self>;
    fn as_ptr(&self) -> *const T;
    /// # Safety
    /// see [std::rc::Weak::from_raw]
    unsafe fn from_raw(ptr: *const T) -> Self;
    fn into_raw(self) -> *const T;
    fn upgrade(&self) -> Option<Self::StrongPointer>;
    fn strong_count(&self) -> usize;
    fn ptr_eq(&self, other: &Self) -> bool;
    fn weak_count(&self) -> usize;
}
