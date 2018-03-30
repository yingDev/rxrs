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

pub struct TapOp<V, Src, Obs, SSO:?Sized+'static>
{
    source: Src,
    obs: Obs,
    PhantomData: PhantomData<(*const V, *const SSO)>
}

pub trait ObservableTap<'a, Src, V, Obs, SSO:?Sized+'static>
{
    fn tap(self, o: Obs) -> TapOp<V, Src, Obs, SSO>;
}

impl<'a, Src, V, Obs, SSO:?Sized+'static> ObservableTap<'a, Src, V, Obs, SSO> for Src where
    for<'x> Obs: ObserverHelper<&'x V>+'a+Clone,
    Src : Observable<'a, V, SSO>
{
    #[inline(always)]
    fn tap(self, o: Obs) -> TapOp<V, Src, Obs, SSO>
    {
        TapOp{ source: self, obs: o, PhantomData }
    }
}

macro_rules! fn_sub {
($s: ty) => {
    fn sub(&self, o: Mss<$s, impl Observer<V> +'a>) -> SubRef
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

impl<'a, V, Src, Obs> Observable<'a, V, Yes> for TapOp<V, Src, Obs, Yes> where
        V: Send+Sync+'static,
        for<'x> Obs: ObserverHelper<&'x V>+Send+Sync+'a+Clone,
        Src : Observable<'a, V, Yes>
{
    fn_sub!(Yes);
}
impl<'a, V:'a, Src, Obs> Observable<'a, V, No> for TapOp<V, Src, Obs, No> where
    for<'x> Obs: ObserverHelper<&'x V>+'a+Clone,
    Src : Observable<'a, V, No>
{
    fn_sub!(No);
}

#[cfg(test)]
mod test
{
    use super::*;
    use observable::*;
    use observable::Observer;
    use op::*;
    use fac;

    #[test]
    fn basic()
    {
        fac::range(0,10).take(3)
            .tap(|v : &_| println!("{}",v))
            .subf(|v| {});
    }
}