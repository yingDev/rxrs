use std::marker::PhantomData;
use std::any::{Any};
use std::rc::Rc;
use std::sync::atomic::AtomicIsize;

use observable::*;
use subref::*;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use observable::*;
use observable::RxNoti::*;
use util::mss::*;

#[derive(Clone)]
pub struct SkipOp<'a, Src, V, SSO:?Sized, SSS:?Sized>
{
    source: Src,
    total: usize,
    PhantomData: PhantomData<(*const V, *const SSO, &'a (), *const SSS)>
}

pub trait ObservableSkip<'a, Src, V, SSO:?Sized, SSS:?Sized> where Src : Observable<'a,V,SSO, SSS>
{
    fn skip(self, total: usize) -> SkipOp<'a, Src, V, SSO, SSS>;
}

impl<'a,Src, V, SSO:?Sized, SSS:?Sized> ObservableSkip<'a, Src, V, SSO, SSS> for Src where Src : Observable<'a, V, SSO, SSS>,
{
    #[inline(always)]
    fn skip(self, total: usize) -> SkipOp<'a, Src, V, SSO, SSS>
    {
        SkipOp{ total, PhantomData, source: self  }
    }
}

macro_rules! fn_sub(($s: ty, $sss:ty) => {
     fn sub(&self, o: Mss<$s, impl Observer<V> +'a>) -> SubRef<$sss>
    {
        let mut count = self.total;
        if count == 0 {
            return self.source.sub(o);
        }

        let sub = InnerSubRef::<$sss>::signal();

        sub.add(self.source.sub_noti(byclone!(sub => move |n|{
            match n {
                Next(v) => {
                    if count == 0 {
                        o.next(v);
                        if o._is_closed(){
                            sub.unsub();
                            return IsClosed::True;
                        }
                    } else { count -= 1; }
                },
                Err(e) => { sub.unsub();o.err(e); }
                Comp => { sub.unsub();o.complete();}
            }
            IsClosed::Default
        })));

        sub.into_subref()
    }
});

impl<'a, Src, V:'a+Send+Sync> Observable<'a,V, Yes, Yes> for SkipOp<'a, Src, V, Yes, Yes> where Src: Observable<'a, V, Yes, Yes>
{
    fn_sub!(Yes, Yes);
}
impl<'a, Src, V:'a+Send+Sync> Observable<'a,V, No, Yes> for SkipOp<'a, Src, V, No, Yes> where Src: Observable<'a, V, No, Yes>
{
    fn_sub!(No, Yes);
}

impl<'a, Src, V:'a+Send+Sync> Observable<'a,V, No, No> for SkipOp<'a, Src, V, No, No> where Src: Observable<'a, V, No, No>
{
    fn_sub!(No, No);
}

#[cfg(test)]
mod test
{
    use super::*;
    use test_fixture::*;

    #[test]
    fn basic()
    {
        let src = SimpleObservable;
        let mut out = 0;
        src.skip(1).subf(|v| out += v);

        assert_eq!(5, out);
    }
}