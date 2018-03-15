use std::sync::Arc;
use observable::Observer;
use observable::Observable;
use subref::SubRef;
use subscriber::SubscriberImpl;
use subscriber::Subscriber;
use std::any::Any;
use std::marker::PhantomData;
use observable::FnCell;
use observable::*;
use observable::RxNoti::*;
use observable::IsClosed;

pub struct TapOp<V, Src, Obs>
{
    source: Src,
    obs: Obs,
    PhantomData: PhantomData<V>
}

pub trait ObservableTap<'x, Src, V:Clone+Send+Sync+'static, Obs> where
        for<'a> Obs: ObserverHelper<&'a V>+Send+Sync+'x+Clone,
        Src : Observable<'x, V>,
        Self: Sized,
{
    fn tap(self, o: Obs) -> TapOp<V, Src, Obs>;
}

impl<'x, Src, V:Clone+Send+Sync+'static, Obs> ObservableTap<'x, Src, V, Obs> for Src where
    V: Send+Sync+'static,
    for<'a> Obs: ObserverHelper<&'a V>+Send+Sync+'x+Clone,
    Src : Observable<'x, V>
{
    fn tap(self, o: Obs) -> TapOp<V, Src, Obs>
    {
        TapOp{ source: self, obs: o, PhantomData }
    }
}

impl<'x, V, Src, Obs> Observable<'x, V> for TapOp<V, Src, Obs> where
        V: Send+Sync+'static,
        for<'a> Obs: ObserverHelper<&'a V>+Send+Sync+'static+Clone,
        Src : Observable<'x, V>
{
    #[inline(always)]
    fn sub(&self, dest: impl Observer<V> + Send + Sync+'x) -> SubRef
    {
        let o = self.obs.clone();
        self.source.sub_noti(move |n| {
            match n {
                Next(v) => {
                    o.next(&v);
                    dest.next(v);
                    if dest._is_closed() { return IsClosed::True; }
                },
                Err(e) => {
                    o.err(e.clone());
                    dest.err(e);
                },
                Comp => {
                    o.complete();
                    dest.complete();
                }
            }
            IsClosed::Default
        })
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    use fac::*;
    use observable::*;
    use observable::Observer;
    use op::*;

    #[test]
    fn basic()
    {
        rxfac::range(0..10)
            .take(5)
            .tap(|v:&i32| println!("{}", v))
            .take(100)
            .subf(|v| {});
    }
}