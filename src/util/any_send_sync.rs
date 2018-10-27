use std::ops::Deref;

pub struct AnySendSync<T>{ t: T }
unsafe impl<T> Send for AnySendSync<T>{}
unsafe impl<T> Sync for AnySendSync<T>{}

impl<T> AnySendSync<T>
{
    #[inline(always)] pub unsafe fn new(t: T) -> AnySendSync<T> { AnySendSync{ t } }
    #[inline(always)] pub fn into_inner(self) -> T { self.t }
}

impl<T> Deref for AnySendSync<T>
{
    type Target = T;
    #[inline(always)] fn deref(&self) -> &T { &self.t }
}