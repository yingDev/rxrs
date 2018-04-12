use std::sync::Arc;
use observable::Observer;
use observable::Observable;
use subref::SubRef;
use std::any::Any;
use std::marker::PhantomData;
use observable::FnCell;
use observable::RxNoti::*;
use util::mss::*;
use observable::*;

pub struct TapOp<V, Src, Obs, SSO:?Sized, SSS:?Sized>
{
    source: Src,
    obs: Obs,
    PhantomData: PhantomData<(*const V, *const SSO, *const SSS)>
}

pub trait ObservableTap<'a, Src, V, Obs, SSO:?Sized, SSS:?Sized>
{
    fn tap(self, o: Obs) -> TapOp<V, Src, Obs, SSO, SSS>;
}

impl<'a, Src, V, Obs, SSO:?Sized, SSS:?Sized> ObservableTap<'a, Src, V, Obs, SSO, SSS> for Src where
    for<'x> Obs: ObserverHelper<&'x V>+'a+Clone,
    Src : Observable<'a, V, SSO, SSS>
{
    #[inline(always)]
    fn tap(self, o: Obs) -> TapOp<V, Src, Obs, SSO, SSS>
    {
        TapOp{ source: self, obs: o, PhantomData }
    }
}

macro_rules! fn_sub {
($s: ty, $sss: ty) => {
    fn sub(&self, o: Mss<$s, impl Observer<V> +'a>) -> SubRef<$sss>
    {
        let tap = self.obs.clone();
        self.source.sub_noti(move |n| {
            match n {
                Next(v) => {
                    tap.next(&v);
                    o.next(v);
                    if o._is_closed() { return IsClosed::True; }
                },
                Err(e) => {
                    tap.err(e.clone());
                    o.err(e);
                },
                Comp => {
                    tap.complete();
                    o.complete();
                }
            }
            IsClosed::Default
        })
    }
};}

impl<'a, V, Src, Obs, SSS:?Sized+'static> Observable<'a, V, Yes, SSS> for TapOp<V, Src, Obs, Yes, SSS> where
        V: Send+Sync+'static,
        for<'x> Obs: ObserverHelper<&'x V>+Send+Sync+'a+Clone,
        Src : Observable<'a, V, Yes, SSS>
{
    fn_sub!(Yes, SSS);
}
impl<'a, V:'a, Src, Obs, SSS:?Sized+'static> Observable<'a, V, No, SSS> for TapOp<V, Src, Obs, No, SSS> where
    for<'x> Obs: ObserverHelper<&'x V>+'a+Clone,
    Src : Observable<'a, V, No, SSS>
{
    fn_sub!(No, SSS);
}

#[cfg(test)]
mod test
{
    use super::*;
    use observable::*;
    use observable::Observer;
    use op::*;
    use test_fixture::*;

    #[test]
    fn basic()
    {
        let src = SimpleObservable;
        src.tap(|v:&_| println!("tap {}", v)).subf(|v| println!("sub {}", v));
    }
}