use crate::traits::*;
use std::{
    pin::Pin,
    rc::{Rc, Weak},
};

/// This marker type implements [RefCountFamily] for [Rc].
///
/// It is used for marking [RefCountFamily] users when creating
/// a new [Rc] pointer.
///
/// ```
/// # use cark_ref_counted::*;
/// # use std::rc::Rc;
/// struct Foo<R: SmartPointerFamily> {
///     name: R::Pointer<String>,
/// }
/// impl<R: SmartPointerFamily> Foo<R> {
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
pub struct RcMark;

impl SmartPointerFamily for RcMark {
    type Pointer<T: ?Sized> = Rc<T>;
    fn new<T>(value: T) -> Self::Pointer<T> {
        Rc::new(value)
    }
}

impl<T: ?Sized> SmartPointer<T> for Rc<T> {
    type Mark = RcMark;
}

impl<T: ?Sized> RefCounted<T> for Rc<T> {
    type StrongMark = RcMark;
    type WeakPointer = Weak<T>;

    fn as_ptr(this: &Self) -> *const T {
        Self::as_ptr(this)
    }

    fn downgrade(this: &Self) -> Weak<T> {
        Self::downgrade(this)
    }

    fn strong_count(this: &Self) -> usize {
        Self::strong_count(this)
    }

    fn weak_count(this: &Self) -> usize {
        Self::weak_count(this)
    }

    fn make_mut(this: &mut Self) -> &mut T
    where
        T: Clone,
    {
        Self::make_mut(this)
    }

    unsafe fn increment_strong_count(ptr: *const T) {
        Self::increment_strong_count(ptr)
    }

    unsafe fn decrement_strong_count(ptr: *const T) {
        Self::decrement_strong_count(ptr)
    }

    fn into_raw(this: Self) -> *const T {
        Self::into_raw(this)
    }

    unsafe fn from_raw(ptr: *const T) -> Self {
        Self::from_raw(ptr)
    }

    fn get_mut(this: &mut Self) -> Option<&mut T> {
        Self::get_mut(this)
    }

    fn new_cyclic<F>(data_fn: F) -> Self
    where
        F: FnOnce(&Weak<T>) -> T,
        T: Sized,
    {
        Self::new_cyclic(data_fn)
    }

    fn pin(value: T) -> Pin<Rc<T>>
    where
        T: Sized,
    {
        Self::pin(value)
    }

    fn try_unwrap(this: Self) -> Result<T, Self>
    where
        T: Sized,
    {
        Self::try_unwrap(this)
    }
}

impl<T: ?Sized> WeakPointer<T> for Weak<T> {
    type StrongMark = RcMark;
    type StrongPointer = Rc<T>;

    fn as_ptr(&self) -> *const T {
        self.as_ptr()
    }

    unsafe fn from_raw(ptr: *const T) -> Self {
        Self::from_raw(ptr)
    }

    fn into_raw(self) -> *const T {
        self.into_raw()
    }

    fn upgrade(&self) -> Option<Rc<T>> {
        self.upgrade()
    }

    fn strong_count(&self) -> usize {
        self.strong_count()
    }

    fn ptr_eq(&self, other: &Self) -> bool {
        self.ptr_eq(other)
    }

    fn weak_count(&self) -> usize {
        self.weak_count()
    }
}
