//pub mod arc;
//pub mod rc;

use std::ops::Deref;
use std::rc::Rc;

pub trait RefCountedMarker {
    type Ptr<T: ?Sized>: RefCounted<T, Family = Self>;
    fn new<T>(value: T) -> Self::Ptr<T>;
}

pub trait RefCounted<T: ?Sized>: Deref<Target = T> + Clone {
    type Family: RefCountedMarker<Ptr<T> = Self>;
}

pub struct RcMark;

impl RefCountedMarker for RcMark {
    type Ptr<T: ?Sized> = Rc<T>;
    fn new<T>(value: T) -> Rc<T> {
        Rc::new(value)
    }
}

impl<T: ?Sized> RefCounted<T> for Rc<T> {
    type Family = RcMark;
}

#[cfg(test)]
mod tests {

    //  use std::rc::Rc;

    //    use crate::rc::RcMarker;

    use super::*;

    struct Foo {
        bar: i32,
    }

    #[test]
    fn wraping_a_simple_type() {
        #[derive(Clone)]
        struct FooContainer<RC: RefCounted<Foo>>(RC);

        impl<RC: RefCounted<Foo>> FooContainer<RC> {
            fn new(foo: RC) -> Self {
                Self(foo)
            }

            fn wrap<T: RefCountedMarker<Ptr<Foo> = RC>>(foo: Foo) -> Self {
                Self(T::new(foo))
            }
        }

        let a = FooContainer::new(Rc::new(Foo { bar: 2 }));
        assert!(a.0.bar == 2);
        let a = FooContainer::wrap::<RcMark>(Foo { bar: 3 });
        assert!(a.0.bar == 3);
        let b = a.clone();
        drop(a);
        assert!(b.0.bar == 3);
        assert!(b.0.bar + 2 == 5);
        let _a: &Foo = b.0.deref();
    }

    // type FnType = dyn Fn(f32) -> f32;

    // struct Bar<RC: RefCounted<FnType>>(RC);

    // impl<RC: RefCounted<FnType>> Bar<RC> {
    //     fn new<M: RefCountedFamily<Ptr<FnType> = RC>>(pre_wrapped: RC) -> Self {
    //         Self(pre_wrapped)
    //     }
    //     //        fn wrap<>
    // }

    // fn add_one(value: f32) -> f32 {
    //     value + 1.0
    // }

    // #[test]
    // fn wraping_a_fn() {
    //     let a = Bar::new::<RcMarker>(Rc::new(add_one));
    //     assert!(a.0(2.0) == 3.0);
    // }

    // #[test]
    // fn wrapping_a_closure() {
    //     //        let a = Bar::new::<RcMarker>
    // }

    //type MyFn = dyn Fn(f32) -> f32;

    // // struct Bar(Rc<MyFn>);

    // struct Bar2<RC: RefCounted<MyFn>>(RC);

    // impl<RC: RefCounted<MyFn>> Bar2<RC> {
    //     fn new(f: RC) -> Self {
    //         Self(f as RC)
    //     }
    //     fn wrap<T: RefCountedFamily<Ptr<MyFn> = RC>>(foo: impl Fn(f32) -> f32) -> Self {
    //         Self::new(T::new(foo))
    //         //            Self(T::new(foo) as RefCounted<MyFn>)
    //     }
    // }

    // // impl<RC: RefCounted<dyn Fn(f32) -> f32>> Bar2<RC> {
    // //     fn new(f: RC) -> Self {
    // //         Self(f)
    // //     }
    // //     // fn w// rap<T: RefCountedFamily<Ptr<>>>(foo: impl Fn(f32) -> f32) -> Self {

    // //     // //            Self(T::new(foo))
    // //     //         }
    // // }

    // fn f1(a: f32) -> f32 {
    //     a * 2.0
    // }

    // #[test]
    // fn bleh() {
    //     // let a = |a: f32| a * 1f32;
    //     // let b = Rc::new(dyn(a));
    //     // let b: Rc<dyn Fn(f32) -> f32> = Rc::new(f1);
    //     //let bar2 = Bar2::new(Rc::new(f1));
    //     //        let bar2 = Bar2::new(Rc::new(f1) as Rc<dyn Fn(f32) -> f32>);
    //     let _ = Bar2::new(Rc::new(f1) as Rc<_>);
    //     let _ = Bar2::new(Rc::new(|a| a + 1.1f32) as Rc<_>);
    //     let _ = Bar2::wrap::<_, RcMarker>(f1);
    //     //        let bar2 = Bar2::new(Rc::new(f1 as Fn(f32) -> f32));
    // }
}
