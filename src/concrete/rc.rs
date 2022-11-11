use crate::traits::*;
use std::rc::Rc;

/// This marker type implements [RefCountFamily] for [Rc].
///
/// It is used for marking [RefCountFamily] users when creating
/// a new [Rc] pointer.
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
pub struct RcMark;

impl RefCountFamily for RcMark {
    type Pointer<T: ?Sized> = Rc<T>;
    fn new<T>(value: T) -> Self::Pointer<T> {
        Rc::new(value)
    }
}

impl<T: ?Sized> RefCounted<T> for Rc<T> {
    type Mark = RcMark;
    fn as_ptr(this: &Self) -> *const T {
        Self::as_ptr(this)
    }
}

impl<T: Clone + ?Sized> RefCountedClone<T> for Rc<T> {
    fn make_mut(this: &mut Self) -> &mut T {
        Self::make_mut(this)
    }
}
