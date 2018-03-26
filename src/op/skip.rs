use std::marker::PhantomData;
use std::any::{Any};
use std::rc::Rc;
use std::sync::atomic::AtomicIsize;

use observable::*;
use subref::SubRef;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use observable::*;
use observable::RxNoti::*;
use util::mss::*;

#[derive(Clone)]
pub struct SkipOp<'a, Src, V, SSO:?Sized+'static>
{
    source: Src,
    total: usize,
    PhantomData: PhantomData<(*const V, *const SSO, &'a ())>
}

pub trait ObservableSkip<'a, Src, V, SSO:?Sized+'static> where Src : Observable<'a,V,SSO>
{
    fn skip(self, total: usize) -> SkipOp<'a, Src, V, SSO>;
}

impl<'a,Src, V, SSO:?Sized+'static> ObservableSkip<'a, Src, V, SSO> for Src where Src : Observable<'a, V, SSO>,
{
    #[inline(always)]
    fn skip(self, total: usize) -> SkipOp<'a, Src, V, SSO>
    {
        SkipOp{ total, PhantomData, source: self  }
    }
}

macro_rules! fn_sub(($s: ty) => {
     fn sub(&self, o: Mss<$s, impl Observer<V> +'a>) -> SubRef
    {
        let mut count = self.total;
        if count == 0 {
            return self.source.sub(o);
        }

        let sub = SubRef::signal();

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
        })).added(sub.clone()));
        sub
    }
});

impl<'a, Src, V:'a+Send+Sync> Observable<'a,V, Yes> for SkipOp<'a, Src, V, Yes> where Src: Observable<'a, V, Yes>
{
    fn_sub!(Yes);
}
impl<'a, Src, V:'a+Send+Sync> Observable<'a,V, No> for SkipOp<'a, Src, V, No> where Src: Observable<'a, V, No>
{
    fn_sub!(No);
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
        src.rx().skip(1).subf(|v| out += v);

        assert_eq!(5, out);
    }
}