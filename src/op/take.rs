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
pub struct TakeOp<'a, Src, V> where Src : Observable<'a, V>
{
    source: Src,
    total: isize,
    PhantomData: PhantomData<(V,&'a())>
}

struct TakeState<'a>
{
    count:AtomicIsize,
    PhantomData: PhantomData<&'a()>
}

pub trait ObservableTake<'a, Src, V> where Src : Observable<'a, V>
{
    fn take(self, total: isize) -> TakeOp<'a, Src, V>;
}

impl<'a, Src, V> ObservableTake<'a, Src, V> for Src where Src : Observable<'a, V>,
{
    fn take(self, total: isize) -> TakeOp<'a, Self, V>
    {
        TakeOp{ total, PhantomData, source: self  }
    }
}

impl<'a, V> SubscriberImpl<V,TakeState<'a>> for Subscriber<'a, V,TakeState<'a>>
{
    fn on_next(&self, v:V)
    {
        if self._state.count.fetch_sub(1, Ordering::SeqCst) == 0
        {
            self.complete();
        }else {
            self._dest.next(v);
        }

        if self._state.count.load(Ordering::SeqCst) == 0
        {
            self.complete();
        }
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

impl<'a, Src, V:'static+Send+Sync> Observable<'a, V> for TakeOp<'a, Src, V> where Src: Observable<'a, V>
{
    fn sub(&self, dest: Arc<Observer<V>+Send+Sync+'a>) -> UnsubRef
    {
        if self.total <= 0 {
            dest.complete();
            return UnsubRef::empty();
        }

        let s = Arc::new(Subscriber::new(TakeState{ count: AtomicIsize::new(self.total), PhantomData}, dest, false));
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
    use fac::*;

    #[test]
    fn basic()
    {
        let result = Arc::new(AtomicIsize::new(0));
        let (a,b,c) = (result.clone(), result.clone(), result.clone());

        let subj = Subject::<isize>::new();

        subj.rx().take(2).subf(move |v| { a.fetch_add(v, Ordering::SeqCst); },
                               (),
                               move | | { c.fetch_add(1000, Ordering::SeqCst); });
        subj.next(1);
        subj.next(1);
        subj.next(1);

        assert_eq!(result.load(Ordering::SeqCst), 1002);
    }

    #[test]
    fn take_over()
    {
        let result = Arc::new(AtomicIsize::new(0));
        let (a,b,c) = (result.clone(), result.clone(), result.clone());


        let subj = Subject::<isize>::new();

        subj.rx().take(100).subf(move |v| { a.fetch_add(v, Ordering::SeqCst); },
                               (),
                               move | | { c.fetch_add(1000, Ordering::SeqCst); });
        subj.next(1);
        subj.next(1);
        subj.next(1);
        subj.complete();

        assert_eq!(result.load(Ordering::SeqCst), 1003);
    }

    #[test]
    fn unstoppable_source()
    {
        let result = Arc::new(AtomicIsize::new(0));
        let (a,b,c) = (result.clone(), result.clone(), result.clone());


        rxfac::range(0..100).take(2).subf(move |v| { a.fetch_add(1, Ordering::SeqCst); },
                                 (),
                                 move | | { b.fetch_add(1000, Ordering::SeqCst); });

        assert_eq!(result.load(Ordering::SeqCst), 1002);
    }

    #[test]
    fn unsub()
    {
        let result = Arc::new(AtomicIsize::new(0));
        let (a,b,c) = (result.clone(), result.clone(), result.clone());

        let subj = Subject::<isize>::new();

        subj.rx().take(2)
            .subf(move |_|{ a.fetch_add(1, Ordering::SeqCst); },
                  (),
                  move | |{ b.fetch_add(1000, Ordering::SeqCst); } )
            .add(move ||{ c.fetch_add(10000, Ordering::SeqCst); } );

        subj.next(1);
        //complete & unsub should run here
        subj.next(1);

        subj.next(1);

        assert_eq!(result.load(Ordering::SeqCst), 11002);
    }

}