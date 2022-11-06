use std::rc::Rc;

use crate::{RefCounted, RefCountedFamily};

pub struct RcMarker;

impl RefCountedFamily for RcMarker {
    type Ptr<T: ?Sized> = Rc<T>;
    fn new<T: ?Sized>(value: T) -> Rc<T> {
        Rc::new(value)
    }
}

impl<T: ?Sized> RefCounted<T> for Rc<T> {
    type Family = RcMarker;
}
