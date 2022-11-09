# cark-ref-counted

A [GATs](https://blog.rust-lang.org/2022/10/28/gats-stabilization.html) powered abstraction for reference counted smart pointers.

## What is it for

Your library needs some kind of a reference counted pointer, but you want to leave the choice between [Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html) and [Rc](https://doc.rust-lang.org/std/rc/struct.Rc.html) to the library user.

## Show me the code

```rust
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

```rust
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


