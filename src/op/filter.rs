use std::marker::PhantomData;
use std::any::Any;
use std::rc::Rc;
use observable::*;
use subscriber::*;
use subref::SubRef;
use std::sync::Arc;

pub struct FilterState<'a, V: 'static+Send+Sync, F: 'a+Send+Sync+Fn(&V)->bool>
{
    pred: Arc<F>,
    PhantomData: PhantomData<(V,&'a())>
}

#[derive(Clone)]
pub struct FilterOp<'a, Src, V: 'static+Send+Sync, F: 'a+Send+Sync+Fn(&V)->bool>
{
    source: Src,
    pred: Arc<F>,
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
    FPred: 'a+Send+Sync+Fn(&V)->bool
{
    fn filter(self, pred: FPred) -> FilterOp<'a, Src, V, FPred>
    {
        FilterOp { source: self, pred: Arc::new(pred), PhantomData }
    }
}

impl<'a, Src, V:Clone+Send+Sync, FPred> Observable<'a, V> for FilterOp<'a, Src, V, FPred> where
    Src : Observable<'a, V>,
    FPred: 'a + Send+Sync+Fn(&V)->bool
{
    fn sub(&self, dest: impl Observer<V> + Send + Sync+'a) -> SubRef
    {
        let s = Arc::new(Subscriber::new(FilterState{ pred: self.pred.clone(), PhantomData }, dest, false));

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
        let r = Arc::new(AtomicUsize::new(0));
        let (r2,r3) = (r.clone(), r.clone());

        let s = Subject::new();
        s.rx().filter(|v| v%2 == 0).take(1).subf(
            move |v| { r.fetch_add(v, Ordering::SeqCst); } ,
            |e|{},
            move | | { r2.fetch_add(100, Ordering::SeqCst); } );
        s.next(1);
        s.next(2);
        s.next(3);

        assert_eq!(r3.load(Ordering::SeqCst), 102);
    }
}