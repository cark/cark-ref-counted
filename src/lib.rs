#![feature(coerce_unsized, unsize)]
use std::{ops::Deref, rc::Rc, sync::Arc};

pub trait RefCounted<T: ?Sized>: Sized + Deref<Target = T> {
    type Mark: RefCountMark<Pointer<T> = Self>;
    fn new<U>(value: U) -> <<Self as RefCounted<T>>::Mark as RefCountMark>::Pointer<U> {
        Self::Mark::new(value)
    }
}

pub trait RefCountMark: Sized {
    type Pointer<T: ?Sized>: RefCounted<T>;
    fn new<T>(value: T) -> Self::Pointer<T>;
}

pub struct RcMark;

impl RefCountMark for RcMark {
    type Pointer<T: ?Sized> = Rc<T>;
    fn new<T>(value: T) -> Self::Pointer<T> {
        Rc::new(value)
    }
}

impl<T: ?Sized> RefCounted<T> for Rc<T> {
    type Mark = RcMark;
}

pub struct ArcMark;

impl RefCountMark for ArcMark {
    type Pointer<T: ?Sized> = Arc<T>;
    fn new<T>(value: T) -> Self::Pointer<T> {
        Arc::new(value)
    }
}

impl<T: ?Sized> RefCounted<T> for Arc<T> {
    type Mark = ArcMark;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::CoerceUnsized;

    #[test]
    fn test_2() {
        // test it with a string
        struct Foo<P: RefCounted<i32>>(P);
        impl<P: RefCounted<i32>> Foo<P> {
            fn wrap(v: i32) -> Self {
                Self(P::new(v))
                // Self(P::Mark::new(v))
            }
        }
        let _a = Foo::<Rc<_>>::wrap(2); // _a: Foo<RcFamily>
        let _a = Foo::<Rc<_>>::wrap(2).0; // _a: Rc<i32>

        // test naked rc with a closure
        struct Bar(Rc<dyn Fn(i32) -> i32>);
        impl Bar {
            fn wrap<T: Fn(i32) -> i32 + 'static>(t: T) -> Self {
                Self(Rc::new(t))
            }
        }
        let _a = Bar::wrap(|a| 1 + a).0; // _a: Rc<dyn Fn(i32) -> i32>

        // test generic pointer with a closure
        let _a = RcMark::new(|a: i32| -> i32 { a + 1 }); // _a: Rc<|i32| -> i32> ... no good
        let _a = Rc::<_>::new(|a: i32| -> i32 { a + 1 }); // _a: Rc<|i32| -> i32> ... no good
        let _a: Rc<dyn Fn(i32) -> i32> = RcMark::new(|a| a + 1); // fixed by coercing to dyn Fn
        let a: Rc<dyn Fn(i32) -> i32> = Rc::<_>::new(|a| a + 1); // fixed by coercing to dyn Fn
        let b: <RcMark as RefCountMark>::Pointer<dyn Fn(i32) -> i32> = RcMark::new(|a| a + 1); // do it generically

        let _v = vec![a, b]; // Same type

        // testing MyRc

        struct Baba<Mark: RefCountMark>(Mark::Pointer<dyn Fn(i32) -> i32>);
        impl<Mark: RefCountMark> Baba<Mark> {
            fn wrap<T: Fn(i32) -> i32 + 'static>(value: T) -> Self
            where
                Mark::Pointer<T>: CoerceUnsized<Mark::Pointer<dyn Fn(i32) -> i32>>,
            {
                Self(Mark::new(value))
            }
        }
        let _a = Baba::<RcMark>::wrap(|a| a + 2).0;

        // do it in a generic struct
        struct Baz<P: RefCounted<dyn Fn(i32) -> i32>>(P);
        impl<P: RefCounted<dyn Fn(i32) -> i32>> Baz<P> {
            fn new(t: P) -> Self {
                Self(t)
            }
            //             fn wrap2(
            //                 value: impl Fn(i32) -> i32 + 'static + CoerceUnsized<dyn Fn(i32) -> i32>,
            //             ) -> Self
            // where
            //                             // <<P as RefCounted<(dyn Fn(i32) -> i32 + 'static)>>::Mark as RefCountMark>::Pointer<
            //                             //     T,
            //                             // >: CoerceUnsized<P>,
            //             {
            //                 Self(P::new(value))
            //             }
            fn wrap<T: Fn(i32) -> i32 + 'static>(value: T) -> Self
            where
                <<P as RefCounted<(dyn Fn(i32) -> i32 + 'static)>>::Mark as RefCountMark>::Pointer<
                    T,
                >: CoerceUnsized<P>,
            {
                Self(P::new(value))
            }
        }

        fn build_it<T, R>(value: T) -> R
        where
            T: Fn(i32) -> i32 + 'static,
            R: RefCounted<dyn Fn(i32) -> i32>,
            <<R as RefCounted<dyn Fn(i32) -> i32>>::Mark as RefCountMark>::Pointer<T>:
                CoerceUnsized<R>,
        {
            R::new(value)
        }
        let _a = build_it::<_, Rc<_>>(|a| a + 2);

        fn build_it_2<T: Fn(i32) -> i32, Mark: RefCountMark>(
            value: T,
        ) -> Mark::Pointer<dyn Fn(i32) -> i32>
        where
            Mark::Pointer<T>: CoerceUnsized<Mark::Pointer<dyn Fn(i32) -> i32>>,
        {
            Mark::new(value)
        }
        let _a = build_it_2::<_, RcMark>(|a| a + 2);

        // type RcBaz = Baz<Rc<dyn Fn(f32) -> f32 + 'static>>;
        // type Fam = RcMark;
        // let _a = Baz::<Rc<_>>::new(Rc::new(|a| a + 2));
        // let _a = Baz::<Rc<_>>::wrap(|a| a + 2);
    }

    // #[test]
    // fn test_1() {
    //     use std::ops::{CoerceUnsized, Deref};
    //     use std::rc::Rc;
    //     // define generic pointers
    //     trait PointerFamily {
    //         type Pointer<T: ?Sized>: Deref<Target = T>; // GAT here
    //         fn new<T>(value: T) -> Self::Pointer<T>;
    //     }

    //     // only implement for Rc
    //     struct RcFamily;

    //     impl PointerFamily for RcFamily {
    //         type Pointer<T: ?Sized> = Rc<T>;
    //         fn new<T>(value: T) -> Self::Pointer<T> {
    //             Rc::new(value)
    //         }
    //     }

    //     // test it with a string
    //     struct Foo<P: PointerFamily>(P::Pointer<String>);
    //     impl<P: PointerFamily> Foo<P> {
    //         fn wrap(v: String) -> Self {
    //             Self(P::new(v))
    //         }
    //     }
    //     let _a = Foo::<RcFamily>::wrap("hello".to_owned()); // _a: Foo<RcFamily>
    //     let _a = Foo::<RcFamily>::wrap("hello".to_owned()).0; // _a: Rc<String>

    //     // test naked rc with a closure
    //     struct Bar(Rc<dyn Fn(i32) -> i32>);
    //     impl Bar {
    //         fn wrap<T: Fn(i32) -> i32 + 'static>(t: T) -> Self {
    //             Self(Rc::new(t))
    //         }
    //     }
    //     let _a = Bar::wrap(|a| 1 + a).0; // _a: Rc<dyn Fn(i32) -> i32>

    //     // test generic pointer with a closure
    //     let _a = RcFamily::new(|a: i32| -> i32 { a + 1 }); // _a: Rc<|i32| -> i32> ... no good
    //     let a: Rc<dyn Fn(i32) -> i32> = RcFamily::new(|a| a + 1); // fixed by coercing to dyn Fn
    //     let b: <RcFamily as PointerFamily>::Pointer<dyn Fn(i32) -> i32> = RcFamily::new(|a| a + 1); // do it generically
    //     let _v = vec![a, b]; // Same type

    //     // do it in a generic struct
    //     struct Baz<P: PointerFamily>(P::Pointer<dyn Fn(i32) -> i32>);
    //     impl<P: PointerFamily> Baz<P> {
    //         fn new(t: P::Pointer<dyn Fn(i32) -> i32>) -> Self {
    //             Self(t)
    //         }
    //         fn wrap<T: Fn(i32) -> i32 + 'static>(t: T) -> Self
    //         where
    //             P::Pointer<T>: CoerceUnsized<P::Pointer<dyn Fn(i32) -> i32>>,
    //         {
    //             Self(P::new(t))
    //         }
    //     }
    //     type RcBaz = Baz<RcFamily>;
    //     type Fam = RcFamily;
    //     let _a = RcBaz::new(Fam::new(|a| a + 2)).0; // _a: Rc<dyn Fn(i32) -> i32>
    //     let _a = RcBaz::wrap(|a| a + 2).0;
    // }

    // #[test]
    // fn test_just_rc_with_closures() {
    //     // start with regular functions
    //     fn add2(v: i32) -> i32 {
    //         v + 2
    //     }
    //     let _a = Rc::new(add2);
    //     let _a: Rc<dyn Fn(i32) -> i32> = Rc::new(add2);
    //     let _a = Rc::new(|a: i32| a + 1);
    //     let _a: Rc<dyn Fn(i32) -> i32> = Rc::new(|a: i32| a + 1);
    // }

    // struct Foo<T: RefCountFamily<i32>>(T);

    // impl<T: RefCountFamily<i32>> Foo<T> {
    //     // fn new(v: i32) -> Self {
    //     //     let b = T::new(v);
    //     //     Self(b)
    //     // }
    //     fn new<M: RefCountFamily<i32, Member<i32> = T>>(v: i32) -> Self {
    //         let b = M::new(v);
    //         Self(b)
    //     }
    // }

    // #[test]
    // fn simple_wrapping() {
    //     let a = 1;
    //     let _b = <Rc<_> as RefCountFamily<_>>::new(a);
    //     //        let _: Foo<Rc<i32>> = Foo::<Rc<i32>>::new(a);
    // }

    // #[test]
    // fn test_pointed() {
    //     let a = 1;
    //     let _a = RcMark::wrap(a);
    //     //      let _b = ArcMark::wrap(a);
    //     fn add2(v: i32) -> i32 {
    //         v + 2
    //     }
    //     let _a: Rc<dyn Fn(i32) -> i32> = RcMark::wrap(add2);
    //     let closed_over = 2;
    //     // let a: Rc<dyn Fn(i32) -> i32> = RcMark::wrap(move |v: i32| closed_over + v);
    //     // let b: Rc<dyn Fn(i32) -> i32> = RcMark::wrap(move |v: i32| v);
    //     let a: Rc<dyn Fn(i32) -> i32> = RcMark::wrap(move |v: i32| closed_over + v);
    //     let b: Rc<dyn Fn(i32) -> i32> = RcMark::wrap(move |v: i32| v);
    //     let closed_over = 3;
    //     let c = RcMark::wrap(move |v: i32| v * closed_over);
    //     //        let c: Pointed<Wrapped<i32> = dyn Fn(i32) -> i32> = RcMark::wrap(move |v: i32| v * v);
    //     let _v = vec![a, b, c];
    // }

    // #[test]
    // fn test_abstract_over_simple() {
    //     struct Foo<T: Pointed>()
    // }
}

// impl<T> RefCountFamily<T> for Arc<T> {
//     type Member<U> = Arc<U>;
// }

// fn into_refcount<Fam, T, U>(val: T) -> Fam::Member<U>
// where
//     Fam: RefCountFamily<T>,
// {
//     Rc::new(T)
// }

// trait RefCounterFamily<T>: Deref<Target = T> {
//     type Member<U>
// }

// pub trait RefCountedMarker {
//     type Ptr<T: ?Sized>: RefCounted<T, Marker = Self>;
//     fn new<T>(value: T) -> Self::Ptr<T>;
// }

// pub trait RefCounted<T: ?Sized>: Deref<Target = T> {
//     type Marker: RefCountedMarker<Ptr<T> = Self>;
//     fn is_ref_counted(&self) -> bool {
//         true
//     }
// }

// pub struct RcMark;

// impl RefCountedMarker for RcMark {
//     type Ptr<T: ?Sized> = Rc<T>;
//     fn new<T>(value: T) -> Self::Ptr<T> {
//         Rc::new(value)
//     }
// }

// impl<T: ?Sized> RefCounted<T> for Rc<T> {
//     type Marker = RcMark;
// }

// pub trait RefCountTraitMarker {
//     type Trait<T: ?Sized>;
//     type Dyn<T: ?Sized> = dyn Self::Trait;
//     type Ptr<T: ?Sized>: RefCounted<dyn T, Marker = Self>;
// }

// // impl RefCountedTraitMarker for RcMark {
// //     type Trait<T: ?Sized> = T;
// //     type Ptr<T: ?Sized> = Rc<dyn T>;
// // }

// // pub struct RcDynMark;
// // impl RefCountedMarker for RcDynMark {
// //     type Ptr<T: ?Sized> = Rc<T>;
// //     fn new(value: impl T) -> Self::Ptr<dyn T> {
// //         Rc::new(value)
// //     }
// // }

// // impl<T: ?Sized> RefCounted<T> for Rc<dyn T> {
// //     type Marker = RcMark;
// // }

// // pub trait RefCountedDyn {
// //     type Type;
// //     type Dyn<D> = dyn D;
// //     type Marker: RefCountedDynMarker<Ptr<T> = Self, Dyn<T> = dyn T>;
// // }

// // pub trait RefCountedDynMarker {
// //     type Ptr<T>: RefCountedDyn<Marker = Self, Type = T>;
// //     type Dyn<T>;
// //     //    fn get_bleh() -> &'static Bleh<A = impl A, B = dyn T>;
// //     //    fn new<T>(value: impl T) -> Self::Ptr<dyn T>;
// // }

// // pub trait Bleh {
// //     type A<T>;
// //     type B<T>;
// //     fn new(value: A) -> B;
// // }

// #[cfg(test)]
// mod tests {

//     //  use std::rc::Rc;

//     //    use crate::rc::RcMarker;

//     use super::*;

//     struct Foo {
//         bar: i32,
//     }

//     #[test]
//     fn wraping_a_simple_type() {
//         #[derive(Clone)]
//         struct FooContainer<RC: RefCounted<Foo>>(RC);

//         impl<RC: RefCounted<Foo>> FooContainer<RC> {
//             fn new(foo: RC) -> Self {
//                 Self(foo)
//             }

//             fn wrap<T: RefCountedMarker<Ptr<Foo> = RC>>(foo: Foo) -> Self {
//                 Self(T::new(foo))
//             }
//         }

//         let a = FooContainer::new(Rc::new(Foo { bar: 2 }));
//         assert!(a.0.bar == 2);
//         let a = FooContainer::wrap::<RcMark>(Foo { bar: 3 });
//         assert!(a.0.bar == 3);
//         let b = a.clone();
//         drop(a);
//         assert!(b.0.bar == 3);
//         assert!(b.0.bar + 2 == 5);
//         let _a: &Foo = b.0.deref();
//     }

//     type FnType = dyn Fn(f32) -> f32 + 'static;

//     struct Bar<RC: RefCounted<dyn Fn(f32) -> f32>>(RC);

//     impl<RC: RefCounted<dyn Fn(f32) -> f32>> Bar<RC> {
//         fn new<M: RefCountedMarker<Ptr<dyn Fn(f32) -> f32> = RC>>(pre_wrapped: RC) -> Self {
//             Self(pre_wrapped)
//         }

//         // fn wrap<M: RefCountedMarker<Ptr<FnType> = RC>>(
//         //     value: impl Fn(f32) -> f32 + 'static,
//         // ) -> Self {
//         //     //let a: RC = M::new(value);
//         //     let a: Rc<FnType> = Rc::new(value);
//         //     Self(a)
//         //     //            todo!()
//         //     //            Self(M::new::<dyn Fn(f32) -> f32>(value))
//         // }

//         // fn wrap<
//         //     //            C: Fn(f32) -> f32 + 'static,
//         //     M: RefCountedMarker<Ptr<dyn Fn(f32) -> f32> = RC>,
//         // >(
//         //     value: impl Fn(f32) -> f32 + 'static,
//         // ) -> Self {
//         //     //            let a: Box<dyn Fn(f32) -> f32 + 'static> = Box::new(value);
//         //     let a = as_rc(value);
//         //     Self(a)
//         // }
//         // fn wrap<C: Fn(f32) -> f32, M: RefCountedMarker<Ptr<FnType> = RC>>(value: C) -> Self {
//         //     Self(M::new::<dyn Fn(f32) -> f32>(value))
//         // }
//         // fn wrap<
//         //     C: Fn(f32) -> f32 + 'static,
//         //     M: RefCountedMarker<Ptr<dyn Fn(f32) -> f32 + 'static> = RC>,
//         // >(
//         //     value: C,
//         // ) -> Self {
//         //     let a: Box<dyn Fn(f32) -> f32 + 'static> = Box::new(value);
//         //     Self(M::new(a))
//         // }
//     }

//     fn add_one(value: f32) -> f32 {
//         value + 1.0
//     }

//     fn as_rc(f: impl Fn(f32) -> f32 + 'static) -> Rc<dyn Fn(f32) -> f32 + 'static> {
//         Rc::new(f)
//     }

//     #[test]
//     fn wraping_a_fn() {
//         let a = Bar::new::<RcMark>(Rc::new(add_one));
//         //this is already a problem ... why do we need to specify the mark ?
//         //let a = Bar::new(Rc::new(add_one));
//         assert!(a.0(2.0) == 3.0);
//         let z = 1.0f32;
//         let a = move |a: f32| a + z;
//         let b = as_rc(a);
//         assert!(b(2.0) == 3.0);

//         //        let rc =
//         //        let f: Fn(f32) -> f32 = add_one;
//         //        let a = Bar::wrap::<fn(f32) -> f32, RcMark>(add_one as _);
//     }

//     #[test]
//     fn wrapping_a_closure_without_abstraction() {
//         let added = 1.0f32;
//         let f = move |x: f32| x + added;
//         let multiplied = 2.0f32;
//         let f2 = move |x: f32| x * multiplied;
//         let rcf: Rc<dyn Fn(f32) -> f32> = Rc::new(f);
//         let rcf2: Rc<dyn Fn(f32) -> f32> = Rc::new(f2);
//         assert!(rcf2(2.0) == 4.0);
//         let _v = vec![rcf, rcf2];
//     }
//     #[test]
//     fn wrapping_a_closure() {
//         let added = 1.0f32;
//         let f = move |x: f32| x + added;
//         let multiplied = 2.0f32;
//         let f2 = move |x: f32| x * multiplied;
//         let rcf: Rc<dyn Fn(f32) -> f32> = RcMark::new(f);
//         let rcf2: Rc<dyn Fn(f32) -> f32> = RcMark::new(f2);
//         assert!(rcf2(2.0) == 4.0);
//         let _v = vec![rcf, rcf2];

//         // again but with an abstract coerce
//         let added = 1.0f32;
//         let f = move |x: f32| x + added;
//         let multiplied = 2.0f32;
//         let f2 = move |x: f32| x * multiplied;
//         let rc = RcMark::new(f);
//         //assert!(rcf.is_ref_counted())
//         // let rcf2: Rc<dyn Fn(f32) -> f32> = RcMark::new(f2);
//         // assert!(rcf2(2.0) == 4.0);
//         // let _v = vec![rcf, rcf2];

//         // let added = 1.0f32;
//         // let f = move |x: f32| x + added;
//         // assert!(f(2.0) == 3.0);
//         // let rcf: Rc<> = RcMark::new(f);
//         // assert!(rcf(2.0) == 3.0);
//         // let multiplied = 2.0f32;
//         // let f2 = move |x: f32| x * multiplied;
//         // let rcf2 = RcMark::new(f2);
//         // assert!(rcf2(2.0) == 4.0);
//         // let _v = vec![rcf, rcf2];
//     }

//     //type MyFn = dyn Fn(f32) -> f32;

//     // // struct Bar(Rc<MyFn>);

//     // struct Bar2<RC: RefCounted<MyFn>>(RC);

//     // impl<RC: RefCounted<MyFn>> Bar2<RC> {
//     //     fn new(f: RC) -> Self {
//     //         Self(f as RC)
//     //     }
//     //     fn wrap<T: RefCountedFamily<Ptr<MyFn> = RC>>(foo: impl Fn(f32) -> f32) -> Self {
//     //         Self::new(T::new(foo))
//     //         //            Self(T::new(foo) as RefCounted<MyFn>)
//     //     }
//     // }

//     // // impl<RC: RefCounted<dyn Fn(f32) -> f32>> Bar2<RC> {
//     // //     fn new(f: RC) -> Self {
//     // //         Self(f)
//     // //     }
//     // //     // fn w// rap<T: RefCountedFamily<Ptr<>>>(foo: impl Fn(f32) -> f32) -> Self {

//     // //     // //            Self(T::new(foo))
//     // //     //         }
//     // // }

//     // fn f1(a: f32) -> f32 {
//     //     a * 2.0
//     // }

//     // #[test]
//     // fn bleh() {
//     //     // let a = |a: f32| a * 1f32;
//     //     // let b = Rc::new(dyn(a));
//     //     // let b: Rc<dyn Fn(f32) -> f32> = Rc::new(f1);
//     //     //let bar2 = Bar2::new(Rc::new(f1));
//     //     //        let bar2 = Bar2::new(Rc::new(f1) as Rc<dyn Fn(f32) -> f32>);
//     //     let _ = Bar2::new(Rc::new(f1) as Rc<_>);
//     //     let _ = Bar2::new(Rc::new(|a| a + 1.1f32) as Rc<_>);
//     //     let _ = Bar2::wrap::<_, RcMarker>(f1);
//     //     //        let bar2 = Bar2::new(Rc::new(f1 as Fn(f32) -> f32));
//     // }
// }
