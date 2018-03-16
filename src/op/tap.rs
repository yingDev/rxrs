use std::sync::Arc;
use observable::Observer;
use observable::Observable;
use subref::SubRef;
use std::any::Any;
use std::marker::PhantomData;
use observable::FnCell;
use observable::*;
use observable::RxNoti::*;
use observable::IsClosed;

pub struct TapOp<'a:'b, 'b, V:'a, Obs> where for<'x> Obs : ObserverHelper<&'x V>+Send+Sync+'a+Clone
{
    source: Arc<Observable<'a, V>+'b+Send+Sync>,
    obs: Obs,
}

pub trait ObservableTap<'a, 'b, V:'a+Send+Sync, Obs>
{
    fn tap(self, o: Obs) -> Arc<Observable<'a,V>+'b+Send+Sync>;
}

impl<'a:'b, 'b, V:'a+Send+Sync, Obs> ObservableTap<'a,'b, V, Obs> for Arc<Observable<'a,V>+'b+Send+Sync> where for<'x> Obs : ObserverHelper<&'x V>+Send+Sync+'a+Clone
{
    fn tap(self, o: Obs) -> Arc<Observable<'a,V>+'b+Send+Sync>
    {
        Arc::new(TapOp{ source: self, obs: o})
    }
}

impl<'a:'b,'b, V:'a, Obs> Observable<'a, V> for TapOp<'a,'b,V, Obs> where for<'x> Obs : ObserverHelper<&'x V>+Send+Sync+'a+Clone
{
    fn sub(&self, dest: Arc<Observer<V>+Send+Sync+'a>) -> SubRef
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
        || rxfac::range(0..10)
            .take(5)
            .tap(|v:&i32| println!("{}", v))
            .take(100)
            .subf(|v| {});
    }
}