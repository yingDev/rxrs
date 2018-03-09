use std::marker::PhantomData;
use std::any::{Any};
use std::rc::Rc;
use std::sync::atomic::AtomicIsize;

use observable::*;
use subscriber::*;
use unsub_ref::UnsubRef;
use std::sync::Arc;
use std::sync::atomic::Ordering;

#[derive(Clone)]
pub struct SkipOp<Src, V> where Src : Observable<V>
{
    source: Src,
    total: isize,
    PhantomData: PhantomData<V>
}

struct SkipState
{
    count:AtomicIsize
}

pub trait ObservableSkip<Src, V> where Src : Observable<  V>
{
    fn skip(self, total: isize) -> SkipOp<Src, V>;
}

impl<Src, V> ObservableSkip<Src, V> for Src where Src : Observable<  V>,
{
    fn skip(self, total: isize) -> SkipOp<Self, V>
    {
        SkipOp{ total, PhantomData, source: self  }
    }
}

impl<V> SubscriberImpl<V,SkipState> for Subscriber<V,SkipState>
{
    fn on_next(&self, v:V)
    {
        if self._state.count.load(Ordering::Acquire) <= 0 {
            if self._dest._is_closed() {
                self.complete();
                return;
            }
            self._dest.next(v);
            if self._dest._is_closed() {
                self.complete();
            }
            return;
        }

        self._state.count.fetch_sub(1, Ordering::SeqCst);
    }

    fn on_err(&self, e:Arc<Any+Send+Sync>)
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

impl<Src, V:'static+Send+Sync> Observable< V> for SkipOp<Src, V> where Src: Observable<V>
{
    fn sub(&self, dest: Arc<Observer<V>+Send+Sync>) -> UnsubRef<'static>
    {
        let s = Arc::new(Subscriber::new(SkipState{ count: AtomicIsize::new(self.total)}, dest, false));
        let sub = self.source.sub(s.clone());
        s.set_unsub(&sub);

        sub
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    use subject::*;

    #[test]
    fn basic()
    {
        let s = Subject::new();
        let result = Arc::new(AtomicIsize::new(0));
        let (a,b) = (result.clone(), result.clone());

        s.rx().skip(1).subf(move |v| { a.fetch_add(v, Ordering::SeqCst); }, (), move || { b.fetch_add(100, Ordering::SeqCst); });
        s.next(1);
        s.next(2);
        s.complete();

        assert_eq!(result.load(Ordering::SeqCst), 102);
    }
}