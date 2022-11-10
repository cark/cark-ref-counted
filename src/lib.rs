#![cfg_attr(feature = "nightly", feature(coerce_unsized))]
/*!
A [GATs](https://blog.rust-lang.org/2022/10/28/gats-stabilization.html) powered abstraction for reference counted smart pointers.

## What is it for

Your library needs some kind of a reference counted pointer, but you want to leave the choice between [Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html) and [Rc](https://doc.rust-lang.org/std/rc/struct.Rc.html) to the library user.

## Show me the code

```
use cark_ref_counted::*;

// My library struct
struct Foo<R: RefCountFamily> {
    name: R::Pointer<String>,
}

impl<R: RefCountFamily> Foo<R> {
    fn name(&self) -> &str {
        &self.name
    }
    fn new(name: &str) -> Self {
        Self {
            // we wrap the string in some ref counted pointer
            name: R::new(name.to_owned()),
        }
    }
}

// RcMark indicates which kind of reference counted
// pointer we want this time
let foo = Foo::<RcMark>::new("John Doe");
assert_eq!(foo.name(), "John Doe");
```

## Some issues are remaining

#### Ergonomics

There are some places where the type inference needs a little bit of help.

```
# use cark_ref_counted::*;
# use std::rc::Rc;
fn wrap<R: RefCountFamily>(value: i32) -> R::Pointer<i32> {
    R::new(value)
}

// Rc doesn't need help
let _a: Rc<i32> = Rc::new(1);

// Type inference doesn't understand which Mark we're using
// let _a: Rc<i32> = wrap(1);

// So we need to specify it
let _a: Rc<i32> = wrap::<RcMark>(1);
```

#### Closures

Rust closures have a great deal of magic attached to them. Wrapping these will require the use of the [coerce_unsized nightly feature](https://doc.rust-lang.org/std/ops/trait.CoerceUnsized.html).

```ignore
# use cark_ref_counted::*;
# use std::rc::Rc;
#![feature(coerce_unsized)]
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
        // about the c parameter, other than it's a closure.
        Self(Mark::new(c))
    }
}
let _a = Foo::<RcMark>::wrap(|a| a + 1).0;

```

We obviously would prefer not to use a nightly feature here. It seems to me that the issue
is not really with GATs but rather with the way rust closures are implemented.

The tentative conclusion to my experiments was that no amount of wrapping would allow this library to alleviate the issue by itself. At some point the closure needs to be passed as a generic parameter, and if we want it to be understood as implementing CoerceUnsized, it needs to be specified right there.

A shot in the dark: Maybe would it be possible for the std team to implement CoerceUnsized for all `Rc<Fn<...>>` to `Rc<dyn Fn<...>>`? And for `Arc` too. While this would still leave an open question for implementers of other reference counted types, the ergonomics would be greatly improved for this use case.

## Thanks

- Reddit user [Eh2406](https://www.reddit.com/user/Eh2406) for pushing.
- Rust programming language forum user [semicoleon](https://users.rust-lang.org/u/semicoleon) for pointing me toward the CoerceUnsized trait.
 */

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
    type Pointer<T: ?Sized>: RefCounted<T>; //Deref<Target = T> + Clone;
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

// RefCounted
pub trait RefCounted<T: ?Sized>: Deref<Target = T> + Clone {
    type Mark: RefCountFamily<Pointer<T> = Self>;
    fn new<U>(value: U) -> <<Self as RefCounted<T>>::Mark as RefCountFamily>::Pointer<U> {
        Self::Mark::new(value)
    }
}

impl<T: ?Sized> RefCounted<T> for Rc<T> {
    type Mark = RcMark;
}

impl<T: ?Sized> RefCounted<T> for Arc<T> {
    type Mark = ArcMark;
}


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
