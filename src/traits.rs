use std::ops::Deref;

/// The trait used to abstract over our concrete pointer types.
///
/// In this library, [RcMark] and [ArcMark] are implementing it.
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
    type Pointer<T: ?Sized>: RefCounted<T>; //Deref<Target = T> + Clone;
    fn new<T>(value: T) -> Self::Pointer<T>;
}

// RefCounted
pub trait RefCounted<T: ?Sized>: Deref<Target = T> + Clone {
    type Mark: RefCountFamily<Pointer<T> = Self>;
    fn new<U>(value: U) -> <<Self as RefCounted<T>>::Mark as RefCountFamily>::Pointer<U> {
        Self::Mark::new(value)
    }
    fn as_ptr(this: &Self) -> *const T;
}

// RefCountedClone
pub trait RefCountedClone<T: Clone + ?Sized>: RefCounted<T> {
    fn make_mut(this: &mut Self) -> &mut T;
}
