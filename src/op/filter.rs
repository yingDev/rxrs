use std::marker::PhantomData;
use std::any::Any;
use std::rc::Rc;
use observable::*;
use subref::SubRef;
use std::sync::Arc;
use observable::*;
use observable::RxNoti::*;

#[derive(Clone)]
pub struct FilterOp<'a:'b, 'b, V: 'static+Send+Sync, F: 'a+Send+Sync+Fn(&V)->bool>
{
    source: Arc<Observable<'a,V>+'b+Send+Sync>,
    pred: F,
    PhantomData: PhantomData<(V,&'a())>
}

pub trait ObservableFilter<'a:'b, 'b, V:'static+Send+Sync, F> where
    F: 'a+Send+Sync+Fn(&V)->bool
{
    fn filter(self, pred: F) -> Arc<Observable<'a, V>+'b+Send+Sync>;
}

impl<'a:'b, 'b, V:'static+Send+Sync, F> ObservableFilter<'a, 'b, V, F> for Arc<Observable<'a, V>+'b+Send+Sync> where
    F: 'a+Clone+Send+Sync+Fn(&V)->bool
{
    fn filter(self, pred: F) -> Arc<Observable<'a, V>+'b+Send+Sync>
    {
        Arc::new(FilterOp { source: self, pred: pred.clone(), PhantomData })
    }
}

impl<'a:'b, 'b, V:Send+Sync, FPred> Observable<'a, V> for FilterOp<'a, 'b, V, FPred> where
    FPred: 'a+Clone+Send+Sync+Fn(&V)->bool
{
    fn sub(&self, o: Arc<Observer<V>+'a+Send+Sync>) -> SubRef
    {
        let f = self.pred.clone();

        self.source.sub_noti(move |n| {
            match n {
                Next(v) => {
                    if f(&v) { o.next(v) };
                    if o._is_closed() { return IsClosed::True; }
                },
                Err(e) => o.err(e),
                Comp => o.complete()
            }
            IsClosed::Default
        })
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    use fac::rxfac;
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