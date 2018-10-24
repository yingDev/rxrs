use std::ops::Deref;

pub struct AnySendSync<T>{ t: T }
unsafe impl<T> Send for AnySendSync<T>{}
unsafe impl<T> Sync for AnySendSync<T>{}

impl<T> AnySendSync<T>
{
    pub unsafe fn new(t: T) -> AnySendSync<T> { AnySendSync{ t } }
    pub fn into_inner(self) -> T { self.t }
}

impl<T> Deref for AnySendSync<T>
{
    type Target = T;
    fn deref(&self) -> &T { &self.t }
}