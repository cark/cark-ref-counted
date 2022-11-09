#![doc = include_str!("../README.md")]
#![cfg_attr(feature = "nightly", feature(coerce_unsized))]
use std::{ops::Deref, rc::Rc, sync::Arc};

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
    type Pointer<T: ?Sized>: Deref<Target = T> + Clone;
    fn new<T>(value: T) -> Self::Pointer<T>;
}

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

// This doesn't buy us much at all

// pub trait RefCounted<T: ?Sized>: Deref<Target = T> {
//     type Mark: RefCountMark<Pointer<T> = Self>;
//     fn new<U>(value: U) -> <<Self as RefCounted<T>>::Mark as RefCountMark>::Pointer<U> {
//         Self::Mark::new(value)
//     }
// }

// impl<T: ?Sized> RefCounted<T> for Rc<T> {
//     type Mark = RcMark;
// }

// impl<T: ?Sized> RefCounted<T> for Arc<T> {
//     type Mark = ArcMark;
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive() {
        fn wrap<R: RefCountFamily>(value: i32) -> R::Pointer<i32> {
            R::new(value)
        }
        let _a = wrap::<RcMark>(1);
        // How can we trick type inference to accept this :
        // let _a: Rc<i32> = wrap(1);
        // Standard Rc doesn't need help
        let _a: Rc<i32> = Rc::new(1);
        // So we need to do this
        let _a: Rc<i32> = wrap::<RcMark>(1);
    }

    #[test]
    fn test_struct() {
        struct Foo {
            name: String,
        }
        impl Foo {
            fn name(&self) -> &str {
                &self.name
            }
        }
        fn wrap<R: RefCountFamily>(value: Foo) -> R::Pointer<Foo> {
            R::new(value)
        }
        let a = wrap::<RcMark>(Foo {
            name: "Bar".to_owned(),
        });
        assert_eq!(a.name(), "Bar");
    }

    #[test]
    fn test_wrapping() {
        struct Foo<R: RefCountFamily> {
            name: R::Pointer<String>,
        }
        impl<R: RefCountFamily> Foo<R> {
            fn name(&self) -> &str {
                &self.name
            }
            fn new(name: &str) -> Self {
                Self {
                    name: R::new(name.to_owned()),
                }
            }
        }
        let foo = Foo::<RcMark>::new("John Doe");
        assert_eq!(foo.name(), "John Doe");
    }

    #[test]
    fn test_closure() {
        let a = RcMark::new(|a: i32| a + 1); // Rc<|i32| -> i32>
        assert_eq!(a(1), 2);
        let _b = RcMark::new(|a: i32| a + 2);
        // a and b do not have the same type, following statement doesn't work
        // let v = vec![a, b];

        let a: Rc<dyn Fn(i32) -> i32> = RcMark::new(|a| a + 1);
        let b: Rc<dyn Fn(i32) -> i32> = RcMark::new(|a| a + 2);
        //coercing to dyn Fn does it
        let _v = vec![a, b]; // same type for a and b

        // Do this generically
        // the c parameter has to be generic here because it isn't Sized
        fn wrap<T: Fn(i32) -> i32, R: RefCountFamily>(c: T) -> R::Pointer<T> {
            R::new(c)
        }
        let _a = wrap::<_, RcMark>(|a| a + 1);
        let _a: Rc<dyn Fn(i32) -> i32> = wrap::<_, RcMark>(|a| a + 1);

        #[cfg(feature = "nightly")]
        {
            use std::ops::CoerceUnsized;

            // Inside a type that needs dyn Fn
            struct Foo<Mark: RefCountFamily>(Mark::Pointer<dyn Fn(i32) -> i32>);
            impl<Mark: RefCountFamily> Foo<Mark> {
                // the c parmeter has to be generic here because it wouldn't be Sized
                fn wrap<T: Fn(i32) -> i32 + 'static>(c: T) -> Self
                where
                    Mark::Pointer<T>: CoerceUnsized<Mark::Pointer<dyn Fn(i32) -> i32>>,
                {
                    // This cannot be done without the coerce_unsized nightly feature
                    // which has to be declared right here, because we cannot assume anything
                    // about the c parameter.
                    Self(Mark::new(c))
                }
            }
            let _a = Foo::<RcMark>::wrap(|a| a + 1).0;
        }

        // #[cfg(feature = "nightly")]
        // {
        //     use std::ops::CoerceUnsized;
        //     // Maybe they can do this kind of thing in std, but for all Fn
        //     impl<T, C: Fn(T) -> T> CoerceUnsized<Rc<dyn Fn(T) -> T>> for std::rc::Rc<C> {}

        //     struct Foo<Mark: RefCountMark>(Mark::Pointer<dyn Fn(i32) -> i32>);
        //     impl<Mark: RefCountMark> Foo<Mark> {
        //         fn wrap<T: Fn(i32) -> i32 + 'static>(c: T) -> Self {
        //             Self(Mark::new(c))
        //         }
        //     }
        //     let _a = Foo::<RcMark>::wrap(|a| a + 1).0;
        //     assert!(false)
        // }
    }
}
