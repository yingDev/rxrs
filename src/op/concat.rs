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
use util::mss::*;
use std::cell::RefCell;

pub struct ConcatOp<'a, V, Src, Next, SrcSSO:?Sized+'static, NextSSO:?Sized+'static, SSO:?Sized+'static>
{
    source : Src,
    next: Next,
    PhantomData: PhantomData<(V,&'a(), *const SSO, *const SrcSSO, *const NextSSO)>
}

pub trait ObservableConcat<'a, V, Next, Src, SrcSSO:?Sized+'static, NextSSO:?Sized+'static, SSO:?Sized+'static>
{
    fn concat(self, next: Next) -> ConcatOp<'a, V, Src, Next, SrcSSO, NextSSO, SSO>;
}

macro_rules! fn_concat {
    ($src:ty, $next:ty, $out:ty) => {
        fn concat(self, next: Next) -> ConcatOp<'a, V, Src, Next, $src, $next, $out>
        {
            ConcatOp{ source: self, next, PhantomData }
        }
    };
}

impl<'a, V,Next,Src> ObservableConcat<'a, V, Next, Src, No, No, No> for Src where
    Src : Observable<'a, V, No>,
    Next: Observable<'a, V, No>+Clone,
{
    fn_concat!(No, No, No);
}

impl<'a, V,Next,Src> ObservableConcat<'a, V, Next, Src, Yes, Yes, Yes> for Src where
    Src : Observable<'a, V, Yes>,
    Next: Observable<'a, V, Yes>+Send+Sync+Clone,
{
    fn_concat!(Yes, Yes, Yes);
}

impl<'a, V,Next,Src> ObservableConcat<'a, V, Next, Src, Yes, No, Yes> for Src where
    Src : Observable<'a, V, Yes>,
    Next: Observable<'a, V, No>+Send+Sync+Clone,
{
    fn_concat!(Yes, No, Yes);
}

impl<'a, V,Next,Src> ObservableConcat<'a, V, Next, Src, No, Yes, Yes> for Src where
    Src : Observable<'a, V, No>,
    Next: Observable<'a, V, Yes>+Clone,
{
    fn_concat!(No, Yes, Yes);
}

macro_rules! fn_sub {
    ($s:ty) => {
        fn sub(&self, o: Mss<$s, impl Observer<V> +'a>) -> SubRef
    {
        let next = self.next.clone();
        let sub = SubRef::signal();

        let mut o:Option<Mss<$s,_>> = Some(o);

        sub.add(self.source.sub_noti(byclone!(sub => move |n| {
            match n {
                Next(v) =>  {
                    let o = o.as_ref().unwrap();
                    if o._is_closed() { sub.unsub(); return IsClosed::True; }
                    else { o.next(v); }
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
                                if o._is_closed() { sub.unsub(); return IsClosed::True; }
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
                    })).added(sub.clone()));

                    return IsClosed::False;
                }
            }
            IsClosed::Default
        })));

        sub
    }
    };
}

impl<'a, V:'a, Src, Next> Observable<'a, V, No> for ConcatOp<'a, V, Src, Next,No, No, No> where
    Src : Observable<'a, V, No>,
    Next: Observable<'a,V, No>+'a+Clone,
{
    fn_sub!(No);
}

impl<'a, V:'static+Send+Sync, Src, Next> Observable<'a, V, Yes> for ConcatOp<'a, V, Src, Next,Yes, Yes, Yes> where
    Src : Observable<'a, V, Yes>,
    Next: Observable<'a,V, Yes>+Send+Sync+'a+Clone,
{
    fn_sub!(Yes);
}

impl<'a, V:'static+Send+Sync, Src, Next> Observable<'a, V, Yes> for ConcatOp<'a, V, Src, Next,No, Yes, Yes> where
    Src : Observable<'a, V, No>,
    Next: Observable<'a,V, Yes>+'a+Clone,
{
    fn_sub!(Yes);
}

impl<'a, V:'static+Send+Sync, Src, Next> Observable<'a, V, Yes> for ConcatOp<'a, V, Src, Next,Yes, No, Yes> where
    Src : Observable<'a, V, Yes>,
    Next: Observable<'a,V, No>+Send+Sync+'static+Clone,
{
    fn_sub!(Yes);
}

#[cfg(test)]
mod test
{
    use super::*;
    use fac;
    use op::*;
    use observable::*;
    use std::sync::atomic::AtomicIsize;
    use scheduler::NewThreadScheduler;
    use test_fixture::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn basic()
    {
        let a = fac::range(0, 3);
        let b = fac::range(3, 3);

        let c = a.concat(b);

        let mut out = 0;
        c.subf(|v| out += v);

        assert_eq!(out, 15)
    }

    #[test]
    fn threaded()
    {
        let a = ThreadedObservable;
        let b = ThreadedObservable;

        let c = a.concat(b);

        let out = Arc::new(AtomicIsize::new(0));
        c.subf(byclone!(out =>move |v| println!("{}", out.fetch_add(v as isize, Ordering::SeqCst))));

        thread::sleep(Duration::from_millis(100));

        assert_eq!(out.load(Ordering::SeqCst), 12);
    }

    #[test]
    fn no_yes()
    {
        let a = SimpleObservable;
        let b = ThreadedObservable;

        let c = a.concat(b);

        let out = Arc::new(AtomicIsize::new(0));
        c.subf(byclone!(out =>move |v| println!("{}", out.fetch_add(v as isize, Ordering::SeqCst))));

        thread::sleep(Duration::from_millis(100));

        assert_eq!(out.load(Ordering::SeqCst), 12);
    }

    #[test]
    fn yes_no()
    {
        let a = ThreadedObservable;
        let b = SimpleObservable;

        let c = a.concat(b);

        let out = Arc::new(AtomicIsize::new(0));
        c.subf(byclone!(out =>move |v| println!("{}", out.fetch_add(v as isize, Ordering::SeqCst))));

        thread::sleep(Duration::from_millis(100));

        assert_eq!(out.load(Ordering::SeqCst), 12);
    }

    #[test]
    fn fac_()
    {
        fac::create(|o|{
            o.next(1);
            o.next(2);
            o.complete();
        }).concat(fac::range(3,2)).subf(|v| println!("{}",v));

        fac::range(0,1).concat(fac::create(|o| {
            o.next(1);
            o.complete();
        })).subf(|v| println!("{}", v));

        fac::range(0,1).concat(fac::just(222)).subf(|v| println!("{}",v));

        fac::empty::<i32>().concat(fac::empty()).subf((
            (),
            (),
            || println!("comp")
        ));
    }
}