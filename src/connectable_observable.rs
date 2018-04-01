use observable::Observable;
use subref::SubRef;
use std::sync::Arc;
use observable::Observer;
use std::marker::PhantomData;
use std::any::Any;
use util::mss::*;

pub struct ConnectableObservable<'a, V, Src, Subj, SSO:?Sized, SSS:?Sized>
{
    source: Src,
    subject: Subj,

    PhantomData: PhantomData<(V,&'a (), *const SSO, *const SSS)>
}
unsafe impl<'a, V, Src, Subj, SSO:?Sized, SSS:?Sized> Send for ConnectableObservable<'a,V,Src,Subj,SSO,SSS> where Src: Send, Subj: Send{}
unsafe impl<'a, V, Src, Subj, SSO:?Sized, SSS:?Sized> Sync for ConnectableObservable<'a,V,Src,Subj,SSO,SSS> where Src: Sync, Subj: Sync{}

impl<'a, V, Src, Subj, SSS:?Sized> ConnectableObservable<'a, V, Src, Subj, Yes, SSS>  where Src: Observable<'a, V, Yes, SSS>, Subj : Observer<V>+Observable<'a, V, Yes, SSS>+Send+Sync+'a
{
    pub fn connect(self) -> SubRef<SSS>
    {
        self.source.sub(Mss::new(self.subject))
    }

    #[inline(always)]
    pub fn new(source: Src, subject: Subj) -> ConnectableObservable<'a, V, Src, Subj, Yes, SSS>
    {
        ConnectableObservable{ source, subject, PhantomData }
    }
}
impl<'a, V, Src, Subj, SSS:?Sized> ConnectableObservable<'a, V, Src, Subj, No, SSS>  where Src: Observable<'a, V, No, SSS>, Subj : Observer<V>+Observable<'a, V, No, SSS>+'a
{
    pub fn connect(self) -> SubRef<SSS>
    {
        self.source.sub(Mss::new(self.subject))
    }

    #[inline(always)]
    pub fn new(source: Src, subject: Subj) -> ConnectableObservable<'a, V, Src, Subj, No, SSS>
    {
        ConnectableObservable{ source, subject, PhantomData }
    }
}

impl<'a, V, Src, Subj, SSS:?Sized> Observable<'a, V, Yes, SSS> for ConnectableObservable<'a, V, Src, Subj, Yes, SSS>  where Src: Observable<'a, V, Yes, SSS>+Send+Sync, Subj : Observer<V>+Observable<'a, V, Yes, SSS>+Send+Sync+'a
{
    #[inline(always)]
    fn sub(&self, o: Mss<Yes, impl Observer<V> +'a>) -> SubRef<SSS>
    {
        self.subject.sub(o)
    }
}

impl<'a, V, Src, Subj, SSS:?Sized> Observable<'a, V, No, SSS> for ConnectableObservable<'a, V, Src, Subj, No, SSS>  where Src: Observable<'a, V, No, SSS>, Subj : Observer<V>+Observable<'a, V, No, SSS>+'a
{
    #[inline(always)]
    fn sub(&self, o: Mss<No, impl Observer<V> +'a>) -> SubRef<SSS>
    {
        self.subject.sub(o)
    }
}