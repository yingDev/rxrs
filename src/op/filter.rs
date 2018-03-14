use std::marker::PhantomData;
use std::any::Any;
use std::rc::Rc;
use observable::*;
use subscriber::*;
use subref::SubRef;
use std::sync::Arc;

pub struct FilterState<'a, V: 'static+Send+Sync, F: 'a+Send+Sync+Fn(&V)->bool>
{
    pred: F,
    PhantomData: PhantomData<(V,&'a())>
}

#[derive(Clone)]
pub struct FilterOp<'a, Src, V: 'static+Send+Sync, F: 'a+Send+Sync+Fn(&V)->bool>
{
    source: Src,
    pred: F,
    PhantomData: PhantomData<(V,&'a())>
}

pub trait ObservableFilter<'a, Src, V:Clone+Send+Sync, FPred> where
    Src : Observable<'a, V>,
    FPred: 'a+Send+Sync+Fn(&V)->bool
{
    fn filter(self, pred: FPred) -> FilterOp<'a, Src, V, FPred>;
}

impl<'a, Src, V:Clone+Send+Sync, FPred> ObservableFilter<'a, Src, V, FPred> for Src where
    Src : Observable<'a, V>,
    FPred: 'a+Clone+Send+Sync+Fn(&V)->bool
{
    fn filter(self, pred: FPred) -> FilterOp<'a, Src, V, FPred>
    {
        FilterOp { source: self, pred: pred.clone(), PhantomData }
    }
}

impl<'a, Src, V:Clone+Send+Sync, FPred> Observable<'a, V> for FilterOp<'a, Src, V, FPred>  where
    Src : Observable<'a, V>,
    FPred: 'a+Clone+Send+Sync+Fn(&V)->bool
{
    fn sub(&self, o: impl Observer<V>+'a+Send+Sync) -> SubRef
    {
        let s = Subscriber::new(FilterState{ pred: self.pred.clone(), PhantomData }, o, false);
        s.do_sub(&self.source)
    }
}

impl<'a, V:Clone+Send+Sync,F, Dest : Observer<V>+Send+Sync+'a> SubscriberImpl<V,FilterState<'a, V, F>> for Subscriber<'a, V,FilterState<'a, V,F>, Dest> where F: 'a+Send+Sync+Fn(&V)->bool
{
    fn on_next(&self, v: V)
    {
        if self._dest._is_closed() {
            self.complete();
        }else if (self._state.pred)(&v) {
            self._dest.next(v);
        }
        if self._dest._is_closed() {
            self.complete();
        }
    }

    fn on_err(&self, e: Arc<Any+Send+Sync>)
    {
        self.do_unsub();
        self._dest.err(e);
    }

    fn on_comp(&self)
    {
        self.do_unsub();
        self._dest.complete();
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    use fac::rxfac;
    use subject::*;
    use op::*;
    use std::cell::RefCell;
    use std::sync::atomic::{Ordering, AtomicUsize};

    #[test]
    fn basic()
    {
        use observable::RxNoti::*;

        let mut sum = 0;

        {
            rxfac::range(0..3).filter(|v| v%2 == 0).sub_noti(|n| match n {
                Next(v) => sum += v,
                Comp => sum += 100,
                _ => {}
            });
        }

        assert_eq!(sum, 102);
    }
}