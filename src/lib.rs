//pub mod arc;
//pub mod rc;

use std::ops::Deref;
use std::rc::Rc;

pub trait RefCountedMarker {
    type Ptr<T: ?Sized>: RefCounted<T, Marker = Self>;
    fn new<T>(value: T) -> Self::Ptr<T>;
}

pub trait RefCounted<T: ?Sized>: Deref<Target = T> {
    type Marker: RefCountedMarker<Ptr<T> = Self>;
    fn is_ref_counted(&self) -> bool {
        true
    }
}

pub struct RcMark;

impl RefCountedMarker for RcMark {
    type Ptr<T: ?Sized> = Rc<T>;
    fn new<T>(value: T) -> Self::Ptr<T> {
        Rc::new(value)
    }
}

impl<T: ?Sized> RefCounted<T> for Rc<T> {
    type Marker = RcMark;
}

// pub struct RcDynMark;
// impl RefCountedMarker for RcDynMark {
//     type Ptr<T: ?Sized> = Rc<T>;
//     fn new(value: impl T) -> Self::Ptr<dyn T> {
//         Rc::new(value)
//     }
// }

// impl<T: ?Sized> RefCounted<T> for Rc<dyn T> {
//     type Marker = RcMark;
// }

// pub trait RefCountedDyn {
//     type Type;
//     type Dyn<D> = dyn D;
//     type Marker: RefCountedDynMarker<Ptr<T> = Self, Dyn<T> = dyn T>;
// }

// pub trait RefCountedDynMarker {
//     type Ptr<T>: RefCountedDyn<Marker = Self, Type = T>;
//     type Dyn<T>;
//     //    fn get_bleh() -> &'static Bleh<A = impl A, B = dyn T>;
//     //    fn new<T>(value: impl T) -> Self::Ptr<dyn T>;
// }

// pub trait Bleh {
//     type A<T>;
//     type B<T>;
//     fn new(value: A) -> B;
// }

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

    type FnType = dyn Fn(f32) -> f32 + 'static;

    struct Bar<RC: RefCounted<dyn Fn(f32) -> f32>>(RC);

    impl<RC: RefCounted<dyn Fn(f32) -> f32>> Bar<RC> {
        fn new<M: RefCountedMarker<Ptr<dyn Fn(f32) -> f32> = RC>>(pre_wrapped: RC) -> Self {
            Self(pre_wrapped)
        }

        // fn wrap<M: RefCountedMarker<Ptr<FnType> = RC>>(
        //     value: impl Fn(f32) -> f32 + 'static,
        // ) -> Self {
        //     //let a: RC = M::new(value);
        //     let a: Rc<FnType> = Rc::new(value);
        //     Self(a)
        //     //            todo!()
        //     //            Self(M::new::<dyn Fn(f32) -> f32>(value))
        // }

        // fn wrap<
        //     //            C: Fn(f32) -> f32 + 'static,
        //     M: RefCountedMarker<Ptr<dyn Fn(f32) -> f32> = RC>,
        // >(
        //     value: impl Fn(f32) -> f32 + 'static,
        // ) -> Self {
        //     //            let a: Box<dyn Fn(f32) -> f32 + 'static> = Box::new(value);
        //     let a = as_rc(value);
        //     Self(a)
        // }
        // fn wrap<C: Fn(f32) -> f32, M: RefCountedMarker<Ptr<FnType> = RC>>(value: C) -> Self {
        //     Self(M::new::<dyn Fn(f32) -> f32>(value))
        // }
        // fn wrap<
        //     C: Fn(f32) -> f32 + 'static,
        //     M: RefCountedMarker<Ptr<dyn Fn(f32) -> f32 + 'static> = RC>,
        // >(
        //     value: C,
        // ) -> Self {
        //     let a: Box<dyn Fn(f32) -> f32 + 'static> = Box::new(value);
        //     Self(M::new(a))
        // }
    }

    fn add_one(value: f32) -> f32 {
        value + 1.0
    }

    fn as_rc(f: impl Fn(f32) -> f32 + 'static) -> Rc<dyn Fn(f32) -> f32 + 'static> {
        Rc::new(f)
    }

    #[test]
    fn wraping_a_fn() {
        let a = Bar::new::<RcMark>(Rc::new(add_one));
        //this is already a problem ... why do we need to specify the mark ?
        //let a = Bar::new(Rc::new(add_one));
        assert!(a.0(2.0) == 3.0);
        let z = 1.0f32;
        let a = move |a: f32| a + z;
        let b = as_rc(a);
        assert!(b(2.0) == 3.0);

        //        let rc =
        //        let f: Fn(f32) -> f32 = add_one;
        //        let a = Bar::wrap::<fn(f32) -> f32, RcMark>(add_one as _);
    }

    #[test]
    fn wrapping_a_closure_without_abstraction() {
        let added = 1.0f32;
        let f = move |x: f32| x + added;
        let multiplied = 2.0f32;
        let f2 = move |x: f32| x * multiplied;
        let rcf: Rc<dyn Fn(f32) -> f32> = Rc::new(f);
        let rcf2: Rc<dyn Fn(f32) -> f32> = Rc::new(f2);
        assert!(rcf2(2.0) == 4.0);
        let _v = vec![rcf, rcf2];
    }
    #[test]
    fn wrapping_a_closure() {
        let added = 1.0f32;
        let f = move |x: f32| x + added;
        let multiplied = 2.0f32;
        let f2 = move |x: f32| x * multiplied;
        let rcf: Rc<dyn Fn(f32) -> f32> = RcMark::new(f);
        let rcf2: Rc<dyn Fn(f32) -> f32> = RcMark::new(f2);
        assert!(rcf2(2.0) == 4.0);
        let _v = vec![rcf, rcf2];

        // again but with an abstract coerce
        let added = 1.0f32;
        let f = move |x: f32| x + added;
        let multiplied = 2.0f32;
        let f2 = move |x: f32| x * multiplied;
        //        let rcf = RcMark::new::<dyn Fn(f32) -> f32>(f);
        //assert!(rcf.is_ref_counted())
        // let rcf2: Rc<dyn Fn(f32) -> f32> = RcMark::new(f2);
        // assert!(rcf2(2.0) == 4.0);
        // let _v = vec![rcf, rcf2];

        // let added = 1.0f32;
        // let f = move |x: f32| x + added;
        // assert!(f(2.0) == 3.0);
        // let rcf: Rc<> = RcMark::new(f);
        // assert!(rcf(2.0) == 3.0);
        // let multiplied = 2.0f32;
        // let f2 = move |x: f32| x * multiplied;
        // let rcf2 = RcMark::new(f2);
        // assert!(rcf2(2.0) == 4.0);
        // let _v = vec![rcf, rcf2];
    }

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
