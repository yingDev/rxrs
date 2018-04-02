use std::rc::Rc;
use std::any::Any;
use observable::*;
use subref::*;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use util::AtomicOption;
use util::ArcCell;
use std::marker::PhantomData;
use observable::*;
use observable::RxNoti::*;
use std::mem;
use util::mss::*;
use std::cell::RefCell;

pub struct ConcatOp<'a, V, Src, Next, SSFlags:?Sized>
{
    source : Src,
    next: Next,
    PhantomData: PhantomData<(V,&'a(), *const SSFlags)>
}

pub trait ObservableConcat<'a, V, Next, Src, SSFlags:?Sized>
{
    fn concat(self, next: Next) -> ConcatOp<'a, V, Src, Next, SSFlags>;
}

macro_rules! fn_concat {
    () => {
        fn concat(self, next: Next) -> ConcatOp<'a, V, Src, Next, SSO, SSS>
        {
            ConcatOp{ source: self, next, PhantomData }
        }
    };
}

impl<'a, V,Next,Src, SSFlags:?Sized> ObservableConcat<'a, V, Next, Src, SSFlags> for Src
{
    fn concat(self, next: Next) -> ConcatOp<'a, V, Src, Next, SSFlags>
    {
        ConcatOp{ source: self, next, PhantomData }
    }
}

macro_rules! fn_sub {
    ($s:ty, $sss:ty) => {
        fn sub(&self, o: Mss<$s, impl Observer<V> +'a>) -> SubRef<$sss>
    {
        let next = self.next.clone();
        let sub = InnerSubRef::<$sss>::signal();

        let mut o:Option<Mss<$s,_>> = Some(o);

        sub.add(self.source.sub_noti(byclone!(sub => move |n| {
            match n {
                Next(v) =>  {
                    let o = o.as_ref().unwrap();
                    o.next(v);
                    if o._is_closed() { sub.unsub(); return IsClosed::True; }
                },
                Err(e) =>  {
                    sub.unsub();
                    o.as_ref().unwrap().err(e);
                },
                Comp => {
                    let o:Mss<$s,_> = o.take().unwrap();
                    sub.add(next.sub_noti(byclone!(sub => move |n| {
                        match n {
                            Next(v) => {
                                o.next(v);
                                if o._is_closed() { sub.unsub();return IsClosed::True; }
                            },
                            Err(e) => {
                                sub.unsub();
                                o.err(e);
                            },
                            Comp => {
                                sub.unsub();
                                o.complete();
                            }
                        }
                        IsClosed::Default
                    })));

                    return IsClosed::False;
                }
            }
            IsClosed::Default
        })));

        sub.into()
    }
    };
}

impl<'a, V:'a, Src, Next> Observable<'a, V, No, No> for ConcatOp<'a, V, Src, Next, (_No, _No)> where
    Src : Observable<'a, V, No, No>,
    Next: Observable<'a,V, No, No>+'a+Clone,
{
    fn_sub!(No, No);
}

impl<'a, V:'static+Send+Sync, Src, Next> Observable<'a, V, Yes, Yes> for ConcatOp<'a, V, Src, Next, (Yes, Yes)> where
    Src : Observable<'a, V, Yes, Yes>,
    Next: Observable<'a,V, Yes, Yes>+Send+Sync+'a+Clone,
{
    fn_sub!(Yes, Yes);
}

impl<'a, V:'static+Send+Sync, Src, Next> Observable<'a, V, No, Yes> for ConcatOp<'a, V, Src, Next, (_No, Yes)> where
    Src : Observable<'a, V, No, Yes>,
    Next: Observable<'a,V, No, Yes>+Send+Sync+'a+Clone,
{
    fn_sub!(No, Yes);
}

#[cfg(test)]
mod test
{
//    use super::*;
//    use fac;
//    use op::*;
//    use observable::*;
//    use std::sync::atomic::AtomicIsize;
//    use scheduler::NewThreadScheduler;
//    use test_fixture::*;
//    use std::thread;
//    use std::time::Duration;
//
//    #[test]
//    fn basic()
//    {
//        let a = fac::range(0, 3);
//        let b = fac::range(3, 3);
//
//        let c = a.concat(b);
//
//        let mut out = 0;
//        c.subf(|v| out += v);
//
//        assert_eq!(out, 15)
//    }
//
//    #[test]
//    fn threaded()
//    {
//        let a = ThreadedObservable;
//        let b = ThreadedObservable;
//
//        let c = a.concat(b);
//
//        let out = Arc::new(AtomicIsize::new(0));
//        c.subf(byclone!(out =>move |v| println!("{}", out.fetch_add(v as isize, Ordering::SeqCst))));
//
//        thread::sleep(Duration::from_millis(100));
//
//        assert_eq!(out.load(Ordering::SeqCst), 12);
//    }
//
//    #[test]
//    fn no_yes()
//    {
//        let a = SimpleObservable;
//        let b = ThreadedObservable;
//
//        let c = a.concat(b);
//
//        let out = Arc::new(AtomicIsize::new(0));
//        c.subf(byclone!(out =>move |v| println!("{}", out.fetch_add(v as isize, Ordering::SeqCst))));
//
//        thread::sleep(Duration::from_millis(100));
//
//        assert_eq!(out.load(Ordering::SeqCst), 12);
//    }
//
//    #[test]
//    fn yes_no()
//    {
//        let a = ThreadedObservable;
//        let b = SimpleObservable;
//
//        let c = a.concat(b);
//
//        let out = Arc::new(AtomicIsize::new(0));
//        c.subf(byclone!(out =>move |v| println!("{}", out.fetch_add(v as isize, Ordering::SeqCst))));
//
//        thread::sleep(Duration::from_millis(100));
//
//        assert_eq!(out.load(Ordering::SeqCst), 12);
//    }
//
//    #[test]
//    fn fac_()
//    {
//        fac::create(|o|{
//            o.next(1);
//            o.next(2);
//            o.complete();
//        }).concat(fac::range(3,2)).subf(|v| println!("{}",v));
//
//        fac::range(0,1).concat(fac::create(|o| {
//            o.next(1);
//            o.complete();
//        })).subf(|v| println!("{}", v));
//
//        fac::range(0,1).concat(fac::just(222)).subf(|v| println!("{}",v));
//
//        fac::empty::<i32>().concat(fac::empty()).subf((
//            (),
//            (),
//            || println!("comp")
//        ));
//    }
}