//use std::rc::Rc;
//use std::any::Any;
//use subscriber::*;
//use observable::*;
//use subref::SubRef;
//use std::sync::atomic::AtomicBool;
//use std::sync::Arc;
//use std::sync::atomic::Ordering;
//use util::AtomicOption;
//use util::ArcCell;
//use std::marker::PhantomData;
//
//
//pub enum ConcatState<'a, V,Next>
//{
//    Cur(CurState<'a, V, Next>), Next
//}
//
//pub struct CurState<'a, V, Next>
//{
//    next: Next,
//    subscriber: AtomicOption<Arc<Observer<V>+Send+Sync+'a>>
//}
//
//
//pub struct ConcatOp<'a, V, Src, Next>
//{
//    source : Src,
//    next: Next,
//    PhantomData: PhantomData<(V,&'a())>
//}
//
//pub trait ObservableConcat<'a, V, Next, Src> where
//    Next: Observable<'a, V>+Send+Sync+Clone,
//    Src : Observable<'a, V>,
//    Self: Sized
//{
//    fn concat(self, next: Next) -> ConcatOp<'a, V, Src, Next>;
//}
//
//impl<'a, V,Next,Src> ObservableConcat<'a, V, Next, Src> for Src where
//    Next: Observable<'a, V>+Send+Sync+Clone,
//    Src : Observable<'a, V>,
//    Self: Sized
//{
//    fn concat(self, next: Next) -> ConcatOp<'a, V, Src, Next>
//    {
//        ConcatOp{ source: self, next: next, PhantomData }
//    }
//}
//
//impl<'a, V:'static+Send+Sync, Src, Next> Observable<'a, V> for ConcatOp<'a, V, Src, Next> where
//    Next: Observable<'a,V>+Send+Sync+Clone+'a,
//    Src : Observable<'a, V>
//{
//    fn sub(&self, dest: impl Observer<V> + Send + Sync+'a) -> SubRef
//    {
//        let cur = CurState{ next: self.next.clone(), subscriber:AtomicOption::new() };
//        let s = Arc::new(Subscriber::new(AtomicOption::new(), dest, false));
//        let s2 = s.clone();
//        cur.subscriber.swap(s2, Ordering::SeqCst);
//        s._state.swap(ConcatState::Cur(cur), Ordering::SeqCst);
//
//        let sub = SubRef::signal();
//        let sub = self.source.sub(s);
//
//        if sub2.disposed() {
//            return SubRef::empty();
//        }
//
//    }
//}
//
//impl<'a, V:'a,Next,Dest> SubscriberImpl<V, AtomicOption<ConcatState<'a, V, Next>>> for Subscriber<'a, V, AtomicOption<ConcatState<'a, V, Next>>,Dest>
//    where Next: 'a+Observable<'a, V>+Send+Sync,
//          Dest : Observer<V>+Send+Sync+'a
//{
//    fn on_next(&self, v: V)
//    {
//        self._dest.next(v);
//    }
//
//    fn on_err(&self, e: Arc<Any+Send+Sync>)
//    {
//        self.do_unsub();
//        self._dest.err(e);
//    }
//
//    fn on_comp(&self)
//    {
//        let state = self._state.take(Ordering::Acquire).unwrap();
//
//        match state {
//            ConcatState::Cur(cur) => {
//                self._state.swap(ConcatState::Next, Ordering::SeqCst);
//                if let Some(sub) = self._sub.take(Ordering::SeqCst) {
//                    self._stopped.store(false, Ordering::SeqCst);
//                    if sub.disposed() {
//                        self._sub.swap(sub, Ordering::SeqCst);
//                        self.complete();
//                        return;
//                    }
//                    self._sub.swap(sub.clone(), Ordering::SeqCst);
//                    sub.add(cur.next.sub(cur.subscriber.take(Ordering::SeqCst).unwrap()));
//                }
//            },
//            ConcatState::Next => {
//                self.do_unsub();
//                self._dest.complete();
//            }
//        }
//
//    }
//}
//
//#[cfg(test)]
//mod test
//{
//    use super::*;
//    use subject::*;
//    use fac::*;
//    use op::*;
//    use observable::*;
//    use std::sync::atomic::AtomicIsize;
//    use scheduler::NewThreadScheduler;
//
//    #[test]
//    fn basic()
//    {
//        let src = rxfac::range(0..10);
//        let even = src.clone().filter(|i:&i32| i % 2 == 0);
//        let odd = src.clone().filter(|i:&i32| i %2 == 1);
//
//        even.concat(odd).concat(rxfac::range(100..105).take(3)).take(100).filter(|v|true).subf(|v| println!("{}",v), (), || println!("comp"));
//
//        //rxfac::timer(100, Some(100), NewThreadScheduler::get()).take(3).concat(rxfac::of(100)).subf(|v| println!("{}",v), (), || println!("comp"));
//
//        //rxfac::range(0..3).concat(Arc::new(rxfac::range(3..6))).subf(|v| println!("{}",v), (), || println!("comp"));
//        ::std::thread::sleep(::std::time::Duration::from_secs(2));
//    }
//
//}