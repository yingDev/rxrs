use std::marker::PhantomData;
use std::any::{Any};
use std::rc::Rc;
use std::sync::atomic::AtomicIsize;

use observable::*;
use subref::SubRef;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use observable::RxNoti::*;

#[derive(Clone)]
pub struct TakeOp<'a, 'b, V>
{
    source: Arc<Observable<'a,V>+'b+Send+Sync>,
    total: isize,
}

pub trait ObservableTake<'a, 'b, V>
{
    fn take(self, total: isize) -> Arc<Observable<'a, V>+'b+Send+Sync>;
}

impl<'a:'b,'b, V:'a> ObservableTake<'a, 'b, V> for Arc<Observable<'a,V>+'b+Send+Sync>
{
    fn take(self, total: isize) -> Arc<Observable<'a, V>+'b+Send+Sync>
    {
        Arc::new(TakeOp{ total, source: self  })
    }
}

impl<'a:'b, 'b, V:'a> Observable<'a, V> for TakeOp<'a,'b, V>
{
    fn sub(&self, dest: Arc<Observer<V> + Send + Sync+'a>) -> SubRef
    {
        if self.total <= 0 {
            dest.complete();
            return SubRef::empty();
        }

        let sub = SubRef::signal();
        let sub2 = sub.clone();
        let mut count = self.total;

        sub.add(self.source.sub_noti(move |n| {
            match n {
                Next(v) => {
                    count -= 1;
                    if count > 0 {
                        dest.next(v);
                        if dest._is_closed() { return IsClosed::True; }
                    }else {
                        dest.next(v);
                        sub2.unsub();
                        dest.complete();
                        return IsClosed::True;
                    }
                },
                Err(e) => {
                    sub2.unsub();
                    dest.err(e);
                },
                Comp => {
                    sub2.unsub();
                    dest.complete()
                }
            }
            IsClosed::Default
        }));

        sub
    }
}

#[cfg(test)]
mod test
{
//    use super::*;
//    use subject::*;
//    use fac::*;
//    use observable::RxNoti::*;
//
//    #[test]
//    fn basic()
//    {
//        let result = Arc::new(AtomicIsize::new(0));
//
//        let subj = Subject::<isize>::new();
//
//        subj.rx().take(2).sub_noti(|n| match n {
//            Next(v) => { result.fetch_add(1, Ordering::SeqCst); },
//            Comp    => { result.fetch_add(1000, Ordering::SeqCst); },
//            _=>{}
//        });
//        subj.next(1);
//        subj.next(1);
//        subj.next(1);
//
//        assert_eq!(result.load(Ordering::SeqCst), 1002);
//    }
//
//    #[test]
//    fn take_over()
//    {
//        let result = Arc::new(AtomicIsize::new(0));
//        let (a,b,c) = (result.clone(), result.clone(), result.clone());
//
//
//        let subj = Subject::<isize>::new();
//
//        subj.rx().take(100).subf((move |v| { a.fetch_add(v, Ordering::SeqCst); },
//                                 (),
//                                 move | | { c.fetch_add(1000, Ordering::SeqCst); }));
//        subj.next(1);
//        subj.next(1);
//        subj.next(1);
//        subj.complete();
//
//        assert_eq!(result.load(Ordering::SeqCst), 1003);
//    }
//
//    #[test]
//    fn unstoppable_source()
//    {
//        let result = Arc::new(AtomicIsize::new(0));
//        let (a,b,c) = (result.clone(), result.clone(), result.clone());
//
//
//        rxfac::range(0..100).take(2).subf((move |v| { a.fetch_add(1, Ordering::SeqCst); },
//                                           (),
//                                           move | | { b.fetch_add(1000, Ordering::SeqCst); }));
//
//        assert_eq!(result.load(Ordering::SeqCst), 1002);
//    }
//
//    #[test]
//    fn unsub()
//    {
//        let result = Arc::new(AtomicIsize::new(0));
//        let (a,b,c) = (result.clone(), result.clone(), result.clone());
//
//        let subj = Subject::<isize>::new();
//
//        subj.rx().take(2)
//            .subf((move |_|{ a.fetch_add(1, Ordering::SeqCst); },
//                   (),
//                   move | |{ b.fetch_add(1000, Ordering::SeqCst); } ))
//            .add(SubRef::from_fn(box move ||{ c.fetch_add(10000, Ordering::SeqCst); }) );
//
//        subj.next(1);
//        //complete & unsub should run here
//        subj.next(1);
//
//        subj.next(1);
//
//        assert_eq!(result.load(Ordering::SeqCst), 11002);
//    }

}