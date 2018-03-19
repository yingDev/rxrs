use std::marker::PhantomData;
use std::any::Any;
use std::rc::Rc;
use observable::*;
use subref::SubRef;
use std::sync::Arc;
use observable::*;
use observable::RxNoti::*;

pub struct FilterState<'a, V, F: 'a+Fn(&V)->bool>
{
    pred: F,
    PhantomData: PhantomData<(V,&'a())>
}

#[derive(Clone)]
pub struct FilterOp<'a, Src, V: 'static, F: 'a+Fn(&V)->bool>
{
    source: Src,
    pred: F,
    PhantomData: PhantomData<(V,&'a())>
}

pub trait ObservableFilter<'a, Src, V, FPred> where
    Src : Observable<'a, V>,
    FPred: 'a+Fn(&V)->bool
{
    fn filter(self, pred: FPred) -> FilterOp<'a, Src, V, FPred>;
}

impl<'a, Src, V:Send+Sync, FPred> ObservableFilter<'a, Src, V, FPred> for Src where
    Src : Observable<'a, V>,
    FPred: 'a+Clone+Fn(&V)->bool
{
    #[inline(always)]
    fn filter(self, pred: FPred) -> FilterOp<'a, Src, V, FPred>
    {
        FilterOp { source: self, pred: pred.clone(), PhantomData }
    }
}

impl<'a, Src, V:Send+Sync, FPred> Observable<'a, V> for FilterOp<'a, Src, V, FPred>  where
    Src : Observable<'a, V>,
    FPred: 'a+Clone+Fn(&V)->bool
{
    #[inline(always)]
    fn sub(&self, o: impl Observer<V>+'a) -> SubRef
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