use std::marker::PhantomData;
use std::any::Any;
use std::rc::Rc;
use observable::*;
use subscriber::*;
use unsub_ref::UnsubRef;
use std::sync::Arc;

pub struct FilterState<V: 'static+Send+Sync+Clone, F: Send+Sync+Fn(&V)->bool>
{
    pred: Arc<F>,
    PhantomData: PhantomData<V>
}

pub struct FilterOp<Src, V: 'static+Send+Sync+Clone, F: Send+Sync+Fn(&V)->bool>
{
    source: Src,
    pred: Arc<F>,
    PhantomData: PhantomData<V>
}

pub trait ObservableFilter<Src, V:Clone+Send+Sync, FPred> where
    Src : Observable<V>,
    FPred: Send+Sync+Fn(&V)->bool
{
    fn filter(self, pred: FPred) -> FilterOp< Src, V, FPred>;
}

impl<Src, V:Clone+Send+Sync, FPred> ObservableFilter<Src, V, FPred> for Src where
    Src : Observable<V>,
    FPred: Send+Sync+Fn(&V)->bool
{
    fn filter(self, pred: FPred) -> FilterOp<Src, V, FPred>
    {
        FilterOp { source: self, pred: Arc::new(pred), PhantomData }
    }
}

impl<Src, V:Clone+Send+Sync, FPred> Observable<V> for FilterOp<Src, V, FPred> where
    Src : Observable<V>,
    FPred: 'static + Send+Sync+Fn(&V)->bool
{
    fn sub(&self, dest: Arc<Observer<V>+Send+Sync>) -> UnsubRef<'static>
    {
        let s = Arc::new(Subscriber::new(FilterState{ pred: self.pred.clone(), PhantomData }, dest, false));
        let sub = self.source.sub(s.clone());
        s.set_unsub(&sub);

        sub
    }
}

impl<V:Clone+Send+Sync,F> Subscriber<V,FilterState<V,F>> where F: Send+Sync+Fn(&V)->bool
{
    #[inline] fn do_pred(&self, v: &V) -> bool { (self._state.pred)(v)  }
}

impl<V:Clone+Send+Sync,F> SubscriberImpl<V,FilterState<V, F>> for Subscriber<V,FilterState<V,F>> where F: Send+Sync+Fn(&V)->bool
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
        self._dest.err(e);
        self.do_unsub();
    }

    fn on_comp(&self)
    {
        self._dest.complete();
        self.do_unsub();
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