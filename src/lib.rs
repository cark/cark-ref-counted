#![cfg_attr(feature = "nightly", feature(coerce_unsized))]
/*!
A [GATs](https://blog.rust-lang.org/2022/10/28/gats-stabilization.html) powered abstraction for reference counted smart pointers.

## What is it for

Your library needs some kind of a reference counted pointer, but you want to leave the choice between [Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html) and [Rc](https://doc.rust-lang.org/std/rc/struct.Rc.html) to the library user.

## Show me the code

```
use cark_ref_counted::*;

// My library struct
struct Foo<R: SmartPointerFamily> {
    name: R::Pointer<String>,
}

impl<R: SmartPointerFamily> Foo<R> {
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
fn wrap<R: SmartPointerFamily>(value: i32) -> R::Pointer<i32> {
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

- Reddit user [Eh2406](https://www.reddit.com/user/Eh2406).
- Rust programming language forum user [semicoleon](https://users.rust-lang.org/u/semicoleon) for pointing me toward the CoerceUnsized trait.
 */

pub mod concrete;
pub mod traits;
//pub use concrete::arc::*;
pub use concrete::rc::*;
pub use traits::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    #[test]
    fn test_try_unwrap() {
        fn actual_test<RC: RefCounted<i32> + std::cmp::PartialEq + std::fmt::Debug>() {
            let x = RC::new(3);
            assert_eq!(RC::try_unwrap(x), Ok(3));
            let x = RC::new(4);
            let _y = RC::clone(&x);
            assert_eq!(*RC::try_unwrap(x).unwrap_err(), 4);
        }
        actual_test::<Rc<_>>()
    }

    #[test]
    fn test_new_cyclic() {
        struct Gadget<P>
        where
            P: SmartPointerFamily,
            P::Pointer<Self>: RefCounted<Self>,
        {
            // TODO: ergonomics
            me: <<P as SmartPointerFamily>::Pointer<Gadget<P>> as RefCounted<Self>>::WeakPointer,
        }
        impl<P> Gadget<P>
        where
            P: SmartPointerFamily,
            P::Pointer<Self>: RefCounted<Self>,
        {
            #[allow(clippy::new_ret_no_self)]
            fn new() -> P::Pointer<Self> {
                P::Pointer::new_cyclic(|me| Gadget { me: me.clone() })
            }
            fn me(&self) -> P::Pointer<Self> {
                self.me.upgrade().unwrap()
            }
        }

        let g = Gadget::<RcMark>::new();
        assert!(Rc::ptr_eq(&g, &g.me()));
    }

    #[test]
    fn test_get_mut() {
        fn actual_test<RC: RefCounted<i32>>() {
            let mut x = RC::new(3);
            *RC::get_mut(&mut x).unwrap() = 4;
            assert_eq!(*x, 4);
            let _y = RC::clone(&x);
            assert!(RC::get_mut(&mut x).is_none());
        }
        actual_test::<Rc<_>>()
    }

    #[test]
    fn test_into_raw() {
        fn actual_test<RC: RefCounted<String>>() {
            let x = RC::new("hello".to_owned());
            let x_ptr = RC::into_raw(x);
            assert_eq!(unsafe { &*x_ptr }, "hello");
        }
        actual_test::<Rc<_>>()
    }

    #[test]
    fn test_increment_strong_count() {
        fn actual_test<RC: RefCounted<i32>>() {
            let five = RC::new(5);
            unsafe {
                let ptr = RC::into_raw(five);
                RC::increment_strong_count(ptr);
                let five = RC::from_raw(ptr);
                assert_eq!(2, RC::strong_count(&five));
            }
        }
        actual_test::<Rc<_>>()
    }

    #[test]
    fn test_decrement_strong_count() {
        fn actual_test<RC: RefCounted<i32>>() {
            let five = RC::new(5);
            unsafe {
                let ptr = RC::into_raw(five);
                RC::increment_strong_count(ptr);

                let five = RC::from_raw(ptr);
                assert_eq!(2, RC::strong_count(&five));
                RC::decrement_strong_count(ptr);
                assert_eq!(1, RC::strong_count(&five));
            }
        }
        actual_test::<Rc<_>>()
    }

    #[test]
    fn test_as_ptr() {
        fn actual_test<RC: RefCounted<String>>() {
            let x = RC::new("hello".to_owned());
            let y = RC::clone(&x);
            let x_ptr = RC::as_ptr(&x);
            assert_eq!(x_ptr, RC::as_ptr(&y));
            assert_eq!(unsafe { &*x_ptr }, "hello");
        }
        actual_test::<Rc<_>>()
    }

    #[test]
    fn test_downgrade() {
        fn actual_test<RC: RefCounted<i32>>() {
            let five = RC::new(5);
            let _weak_five = RC::downgrade(&five);
            assert_eq!(1, RC::weak_count(&five));
        }
        actual_test::<Rc<_>>()
    }

    #[test]
    fn test_weak_upgrade() {
        fn actual_test<RC: RefCounted<i32>>() {
            let five = RC::new(5);
            let weak_five = RC::downgrade(&five);
            let strong_five = weak_five.upgrade();
            assert!(strong_five.is_some());
            drop(strong_five);
            drop(five);
            assert!(weak_five.upgrade().is_none());
        }
        actual_test::<Rc<_>>();
    }

    #[test]
    fn test_weak_ptr_eq() {
        fn actual_test<RC: RefCounted<i32>>() {
            let first_rc = RC::new(5);
            let first = RC::downgrade(&first_rc);
            let second = RC::downgrade(&first_rc);
            assert!(first.ptr_eq(&second));
            let third_rc = RC::new(5);
            let third = RC::downgrade(&third_rc);
            assert!(!first.ptr_eq(&third));
        }
        actual_test::<Rc<_>>();
    }

    #[test]
    fn test_weak_as_ptr() {
        fn actual_test<RC: RefCounted<String>>() {
            use std::ptr;
            let strong = RC::new("hello".to_owned());
            let weak = RC::downgrade(&strong);
            assert!(ptr::eq(&*strong, weak.as_ptr()));
            assert_eq!("hello", unsafe { &*weak.as_ptr() });
            drop(strong)
        }
        actual_test::<Rc<_>>()
    }

    #[test]
    fn test_weak_from_raw() {
        fn actual_test2<RC: RefCounted<String>>() {
            let strong = RC::new("hello".to_owned());
            let raw1 = RC::downgrade(&strong).into_raw();
            let raw2 = RC::downgrade(&strong).into_raw();
            assert_eq!(2, RC::weak_count(&strong));
            assert_eq!(
                "hello",
                &*unsafe { RC::WeakPointer::from_raw(raw1) }
                    .upgrade()
                    .unwrap()
            );
            assert_eq!(1, RC::weak_count(&strong));
            drop(strong);
            assert!(unsafe { RC::WeakPointer::from_raw(raw2) }
                .upgrade()
                .is_none());
        }
        actual_test2::<Rc<_>>();
    }

    #[test]
    fn test_make_mut() {
        fn actual_test<Mark>()
        where
            Mark: SmartPointerFamily,
            Mark::Pointer<i32>: RefCounted<i32>,
        {
            let mut data = Mark::new(5i32);
            *Mark::Pointer::make_mut(&mut data) += 1;
            let mut other_data = Mark::Pointer::clone(&data);
            *Mark::Pointer::make_mut(&mut data) += 1;
            // other incantation, same thing
            *Mark::Pointer::make_mut(&mut data) += 1;
            *Mark::Pointer::make_mut(&mut other_data) *= 2;
            assert_eq!(*data, 8);
            assert_eq!(*other_data, 12);
        }
        actual_test::<RcMark>();

        fn actual_test2<RC: RefCounted<i32>>() {
            let mut data = RC::new(5);
            *RC::make_mut(&mut data) += 1;
            let mut other_data = RC::clone(&data);
            *RC::make_mut(&mut data) += 1;
            *RC::make_mut(&mut data) += 1;
            *RC::make_mut(&mut other_data) *= 2;
            assert_eq!(*data, 8);
            assert_eq!(*other_data, 12);
        }
        actual_test2::<Rc<_>>();
    }

    #[test]
    fn test_primitive() {
        fn wrap<R>(value: i32) -> R::Pointer<i32>
        where
            R: SmartPointerFamily,
            R::Pointer<i32>: RefCounted<i32>,
        {
            R::new(value)
        }
        let _a = wrap::<RcMark>(1);
        // cannot let the inference do its job
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
        fn wrap<R: RefCounted<Foo>>(value: Foo) -> R {
            R::new(value)
        }
        let a = wrap::<Rc<_>>(Foo {
            name: "Bar".to_owned(),
        });
        assert_eq!(a.name(), "Bar");
    }

    #[test]
    fn test_wrapping() {
        struct Foo<R: RefCounted<String>> {
            name: R,
        }
        impl<R: RefCounted<String>> Foo<R> {
            fn name(&self) -> &str {
                &self.name
            }
            fn new(name: &str) -> Self {
                Self {
                    name: R::new(name.to_owned()),
                }
            }
        }
        let foo = Foo::<Rc<_>>::new("John Doe");
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
        fn wrap<T: Fn(i32) -> i32, R: RefCounted<T>>(c: T) -> R {
            R::new(c)
        }
        let _a = wrap::<_, Rc<_>>(|a| a + 1);
        let _a: Rc<dyn Fn(i32) -> i32> = wrap::<_, Rc<_>>(|a| a + 1);

        #[cfg(feature = "nightly")]
        {
            use std::ops::CoerceUnsized;

            // Inside a type that needs dyn Fn
            struct Foo<R: SmartPointerFamily>(R::Pointer<dyn Fn(i32) -> i32>)
            where
                R::Pointer<dyn Fn(i32) -> i32>: RefCounted<dyn Fn(i32) -> i32>;
            impl<R: SmartPointerFamily> Foo<R>
            where
                R::Pointer<dyn Fn(i32) -> i32>: RefCounted<dyn Fn(i32) -> i32>,
            {
                // the c parmeter has to be generic here because it wouldn't be Sized
                fn wrap<T: Fn(i32) -> i32 + 'static>(c: T) -> Self
                where
                    R::Pointer<T>: CoerceUnsized<R::Pointer<dyn Fn(i32) -> i32>>,
                {
                    // This cannot be done without the coerce_unsized nightly feature
                    // which has to be declared right here, because we cannot assume anything
                    // about the c parameter.
                    Self(R::new(c))
                }
            }
            let _a = Foo::<RcMark>::wrap(|a| a + 1).0;
        }

        // #[cfg(feature = "nightly")]
        // {
        //     use std::ops::CoerceUnsized;

        //     // Inside a type that needs dyn Fn
        //     struct Foo<R: RefCounted<dyn Fn(i32) -> i32>>(R);
        //     impl<R: RefCounted<dyn Fn(i32) -> i32>> Foo<R> {
        //         // the c parmeter has to be generic here because it wouldn't be Sized
        //         fn wrap<T: Fn(i32) -> i32 + 'static>(c: T) -> Self
        //         where
        //             R: CoerceUnsized<RefCounted<dyn Fn(i32) -> i32, >>,
        //         {
        //             // This cannot be done without the coerce_unsized nightly feature
        //             // which has to be declared right here, because we cannot assume anything
        //             // about the c parameter.
        //             Self(R::new(c))
        //         }
        //     }
        //     let _a = Foo::<RcMark>::wrap(|a| a + 1).0;
        // }

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
