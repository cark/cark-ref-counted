use std::{ops::Deref, pin::Pin};

/// The trait used to abstract over our concrete pointer types.
///
/// In this library, [crate::concrete::rc::RcMark] and [crate::concrete::arc::ArcMark] are implementing it.
///
/// ```
/// # use cark_ref_counted::*;
/// # use std::rc::Rc;
/// struct Foo<R: RefCountFamily> {
///     name: R::Pointer<String>,
/// }
/// impl<R: RefCountFamily> Foo<R> {
///     fn name(&self) -> &str {
///         &self.name
///     }
///     fn new(name: &str) -> Self {
///         Self {
///             name: R::new(name.to_owned()),
///         }
///     }
/// }
/// let foo = Foo::<RcMark>::new("John Doe");
/// assert_eq!(foo.name(), "John Doe");
/// ```
pub trait RefCountFamily {
    type Pointer<T: ?Sized>: RefCounted<
        T,
        Mark = Self,
        WeakMark = Self::WeakMark<T>,
        WeakPointer = Self::WeakPointer<T>,
    >;
    type WeakPointer<T: ?Sized>: WeakPointer<
        T,
        Mark = Self::WeakMark<T>,
        StrongMark = Self,
        StrongPointer = Self::Pointer<T>,
    >;
    type WeakMark<T: ?Sized>: WeakFamily<
        StrongMark<T> = Self,
        StrongPointer<T> = Self::Pointer<T>,
        Pointer<T> = Self::WeakPointer<T>,
    >;
    fn new<T>(value: T) -> Self::Pointer<T>;
}

pub trait RefCounted<T: ?Sized>: Deref<Target = T> + Clone {
    type Mark: RefCountFamily<
        Pointer<T> = Self,
        WeakPointer<T> = Self::WeakPointer,
        WeakMark<T> = Self::WeakMark,
    >;
    type WeakMark: WeakFamily<
        Pointer<T> = Self::WeakPointer,
        StrongPointer<T> = Self,
        StrongMark<T> = Self::Mark,
    >;
    type WeakPointer: WeakPointer<
        T,
        Mark = Self::WeakMark,
        StrongMark = Self::Mark,
        StrongPointer = Self,
    >;
    fn new<U>(value: U) -> <Self::Mark as RefCountFamily>::Pointer<U> {
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

pub trait WeakFamily {
    type Pointer<T: ?Sized>: WeakPointer<
        T,
        Mark = Self,
        StrongMark = Self::StrongMark<T>,
        StrongPointer = Self::StrongPointer<T>,
    >;
    type StrongPointer<T: ?Sized>: RefCounted<
        T,
        Mark = Self::StrongMark<T>,
        WeakMark = Self,
        WeakPointer = Self::Pointer<T>,
    >;
    type StrongMark<T: ?Sized>: RefCountFamily<
        Pointer<T> = Self::StrongPointer<T>,
        WeakMark<T> = Self,
        WeakPointer<T> = Self::Pointer<T>,
    >;
    fn new<T>() -> Self::Pointer<T>;
}

pub trait WeakPointer<T: ?Sized>: Clone {
    type Mark: WeakFamily<
        Pointer<T> = Self,
        StrongPointer<T> = Self::StrongPointer,
        StrongMark<T> = Self::StrongMark,
    >;
    type StrongMark: RefCountFamily<
        Pointer<T> = Self::StrongPointer,
        WeakPointer<T> = Self,
        WeakMark<T> = Self::Mark,
    >;
    type StrongPointer: RefCounted<
        T,
        Mark = Self::StrongMark,
        WeakMark = Self::Mark,
        WeakPointer = Self,
    >;
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
