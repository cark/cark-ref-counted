use std::sync::Arc;

use crate::{RefCounted, RefCountedFamily};

pub struct ArcFamily;

impl RefCountedFamily for ArcFamily {
    type Ptr<T: ?Sized> = Arc<T>;
    fn new<T>(value: T) -> Arc<T> {
        Arc::new(value)
    }
}

impl<T> RefCounted<T> for Arc<T> {
    type Family = ArcFamily;
}
