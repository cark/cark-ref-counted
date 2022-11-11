use crate::traits::*;
use std::sync::Arc;

/// This marker type implements [RefCountFamily] for [Arc].
///
/// It is used for marking [RefCountFamily] users when creating
/// a new [Arc] pointer.
///
/// ```
/// # use cark_ref_counted::*;
/// # use std::sync::Arc;
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
/// let foo = Foo::<ArcMark>::new("John Doe");
/// assert_eq!(foo.name(), "John Doe");
/// ```
pub struct ArcMark;

impl RefCountFamily for ArcMark {
    type Pointer<T: ?Sized> = Arc<T>;
    fn new<T>(value: T) -> Self::Pointer<T> {
        Arc::new(value)
    }
}

impl<T: ?Sized> RefCounted<T> for Arc<T> {
    type Mark = ArcMark;
    fn as_ptr(this: &Self) -> *const T {
        Self::as_ptr(this)
    }
}

impl<T: Clone + ?Sized> RefCountedClone<T> for Arc<T> {
    fn make_mut(this: &mut Self) -> &mut T {
        Self::make_mut(this)
    }
}
