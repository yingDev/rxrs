use std::rc::Rc;
use std::any::Any;
use observable::*;
use subref::SubRef;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use util::AtomicOption;
use util::ArcCell;
use std::marker::PhantomData;
use observable::*;
use observable::RxNoti::*;
use std::mem;

pub struct ConcatOp<'a, 'b, V>
{
    source : Arc<Observable<'a,V>+'b+Send+Sync>,
    next: Arc<Observable<'a,V>+'b+Send+Sync>,
}

pub trait ObservableConcat<'a, 'b, V>
{
    fn concat(self, next: Arc<Observable<'a,V>+'b+Send+Sync>) -> Arc<Observable<'a, V>+'b+Send+Sync>;
}

impl<'a:'b, 'b, V:'a+Send+Sync> ObservableConcat<'a, 'b, V> for Arc<Observable<'a, V>+'b+Send+Sync>
{
    #[inline(always)]
    fn concat(self, next: Arc<Observable<'a,V>+'b+Send+Sync>) -> Arc<Observable<'a, V>+'b+Send+Sync>
    {
        Arc::new(ConcatOp{ source: self, next: next })
    }
}

impl<'a:'b, 'b, V:'a+Send+Sync> Observable<'a, V> for ConcatOp<'a,'b, V>
{
    #[inline(always)]
    fn sub(&self, dest: Arc<Observer<V>+'a+Send+Sync>) -> SubRef
    {
        let next = self.next.clone();

        let sub = SubRef::signal();
        let sub2 = sub.clone();

        //let mut dest = Some(dest);

        sub.add(self.source.sub_noti(move |n| {

//            match n {
//                Next(v) =>  {
//                    dest.as_ref().unwrap().next(v);
//                    if dest.as_ref().unwrap()._is_closed() { return IsClosed::True; }
//                },
//                Err(e) =>  {
//                    dest.as_ref().unwrap().err(e);
//                    sub2.unsub();
//                },
//                Comp => {
//                    if sub2.disposed() {
//                        dest.as_ref().unwrap().complete();
//                    }else {
//                        let dest = mem::replace(&mut dest, None).unwrap();
//                        let sub3 = sub2.clone();
//                        sub2.add(next.sub_noti(move |n| {
//                            match n {
//                                Next(v) => {
//                                    dest.next(v);
//                                    if dest._is_closed() { return IsClosed::True; }
//                                },
//                                Err(e) => {
//                                    dest.err(e);
//                                    sub3.unsub();
//                                },
//                                Comp => {
//                                    dest.complete();
//                                    sub3.unsub();
//                                }
//                            }
//                            IsClosed::Default
//                        }));
//                    }
//                }
//            }
            IsClosed::Default
        }));

        sub
    }
}


#[cfg(test)]
mod test
{
    use super::*;
    use subject::*;
    use fac::*;
    use op::*;
    use observable::*;
    use std::sync::atomic::AtomicIsize;
    use scheduler::NewThreadScheduler;

    #[test]
    fn basic()
    {
        let src = rxfac::range(0..10);
        let even = src.clone().filter(|i:&i32| i % 2 == 0);
        let odd = src.clone().filter(|i:&i32| i %2 == 1);

        even.concat(odd).concat(rxfac::range(100..105).take(3)).take(100).filter(|v|true).subf((
            |v| println!("{}",v),
            (),
            || println!("comp")));

        //rxfac::timer(100, Some(100), NewThreadScheduler::get()).take(3).concat(rxfac::of(100)).subf(|v| println!("{}",v), (), || println!("comp"));

        //rxfac::range(0..3).concat(Arc::new(rxfac::range(3..6))).subf(|v| println!("{}",v), (), || println!("comp"));
        ::std::thread::sleep(::std::time::Duration::from_secs(2));
    }

    #[test]
    fn unsub()
    {
        let mut x = 0;
        {
            let s = Subject::anew();
            s.rx().subf(|v| x += v).unsub();
            s.next(1);
        }
        assert_eq!(0, x);

    }

}