use std::ops::CoerceUnsized;
use std::ops::Deref;
use std::marker::Unsize;
use std::mem;
use std::sync::Arc;
use std::any::Any;

pub trait No {}
#[derive(Clone)]
pub struct Yes;
impl Yes{}
impl No for Yes{}
pub struct _No;
impl No for _No{}

///'Maybe Send+Sync'
pub struct Mss<S:?Sized, T>
{
    t: T,
    s: *const S
}
unsafe impl<T> Send for Mss<Yes, T>{}
unsafe impl<T> Sync for Mss<Yes, T>{}
impl<T> CoerceUnsized<Mss<No, T>> for Mss<Yes, T>{}
impl<T> CoerceUnsized<Mss<No, T>> for Mss<_No, T>{}

impl<S:?Sized, T> Mss<S,T>
{
    pub fn into_inner(self) -> T
    {
        self.t
    }

    pub fn into_boxed<Trait:?Sized>(self) -> Mss<S, Box<Trait>> where T : Unsize<Trait>
    {
        Mss{ t: Box::new(self.t) as Box<Trait>, s: self.s }
    }
}

impl<T: Send+Sync> Mss<Yes, T>
{
    pub fn yes(t: T) -> Mss<Yes, T> {
        Mss{ t, s: ::std::ptr::null() }
    }
}

impl<T> Mss<No, T>
{
    pub fn no(t: T) -> Mss<No, T> {
        Mss{ t, s: &_No }
    }
}

pub trait MssNew<T, S:?Sized>
{
    fn new(t:T) -> Mss<S,T>;
}
impl<T> MssNew<T, No> for Mss<No, T>
{
    fn new(t:T) -> Mss<No, T>
    {
        Mss::no(t)
    }
}
impl<T:Send+Sync> MssNew<T, Yes> for Mss<Yes, T>
{
    fn new(t:T) -> Mss<Yes, T>
    {
        Mss::yes(t)
    }
}

impl<S:?Sized,T> Deref for Mss<S, T>
{
    type Target = T;
    fn deref(&self) -> &T
    {
        &self.t
    }
}

#[macro_export]
macro_rules! mss(($t: expr) => {
    Mss::new($t)
});
