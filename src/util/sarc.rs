use util::traicks::*;
use std::sync::Arc;
use std::marker::PhantomData;
use std::ops::CoerceUnsized;
use std::marker::Unsize;
use std::ops::Deref;

pub struct Sarc<S, T:?Sized> where S : YesNo
{
    t: Arc<T>,
    s: S
}
impl<S, T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Sarc<S, U>> for Sarc<S, T> where S : YesNo{}
unsafe impl<T:?Sized> Send for Sarc<Yes, T>{}
unsafe impl<T:?Sized> Sync for Sarc<Yes, T>{}
impl<S: YesNo,T:?Sized> Clone for Sarc<S,T>
{
    fn clone(&self) -> Sarc<S, T>
    {
        Sarc{ t: self.t.clone(), s: Default::default() }
    }
}

impl<S:YesNo,T:?Sized> Sarc<S,T>
{
    pub fn as_ref(selv: &Sarc<S,T>) -> &T { Arc::as_ref(&selv.t) }

    pub fn from(t: Arc<T>) -> Sarc<S,T>
    {
        Sarc{ t, s: Default::default() }
    }
}


impl <S: YesNo, T:Sized> Sarc<S,T>
{
    pub fn new(t: T) -> Sarc<S,T>
    {
        Sarc{ t: Arc::new(t), s: Default::default()}
    }
}

pub trait SendSyncInfo
{
    type SSI: YesNo;
}

impl<S:YesNo, T:?Sized> Deref for Sarc<S, T>
{
    type Target = T;
    fn deref(&self) -> &T
    {
        &*self.t
    }
}

#[cfg(test)]
mod test
{
    use super::*;

}