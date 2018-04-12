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
use scheduler::get_sync_context;

//todo: simplify

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

        sub.into_subref()
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
    Next: Observable<'a,V, No, Yes>+'a+Clone,
{
    fn_sub!(No, Yes);
}

impl<'a, V:'static+Send+Sync, Src, Next> Observable<'a, V, Yes, No> for ConcatOp<'a, V, Src, Next, (_No, _No, Yes, Yes)> where
    Src : Observable<'a, V, No, No>,
    Next: Observable<'a,V, Yes, Yes>+'a+Clone,
{
    fn sub(&self, o: Mss<Yes, impl Observer<V> +'a>) -> SubRef<No>
    {
        let next = self.next.clone();
        let sub = InnerSubRef::<No>::signal();

        let first = InnerSubRef::<No>::signal();
        let second = InnerSubRef::<Yes>::signal();

        sub.add(first.clone());
        sub.addss(second.clone().into_subref());

        let mut o:Option<Mss<Yes,_>> = Some(o);

        first.add(self.source.sub_noti(byclone!(first => move |n| {
            match n {
                    Next(v) =>  {
                        let o = o.as_ref().unwrap();
                        o.next(v);
                        if o._is_closed() { first.unsub(); return IsClosed::True; }
                    },
                    Err(e) =>  {
                        first.unsub();
                        o.as_ref().unwrap().err(e);
                    },
                    Comp => {
                        first.unsub();
                        if second.disposed() { return IsClosed::True; }

                        let o:Mss<Yes,_> = o.take().unwrap();
                        second.add(next.sub_noti(byclone!(second => move |n| {
                        match n {
                            Next(v) => {
                                o.next(v);
                                if o._is_closed() { second.unsub();return IsClosed::True; }
                            },
                            Err(e) => {
                                second.unsub();
                                o.err(e);
                            },
                            Comp => {
                                second.unsub();
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

        sub.into_subref()
    }
}

impl<'a, V:'static+Send+Sync, Src, Next> Observable<'a, V, No, No> for ConcatOp<'a, V, Src, Next, (_No, Yes, _No, _No)> where
    Src : Observable<'a, V, No, Yes>,
    Next: Observable<'a,V, No, No>+'a+Clone,
{
    fn sub(&self, o: Mss<No, impl Observer<V> +'a>) -> SubRef<No>
    {
        let next = self.next.clone();
        let sub = InnerSubRef::<No>::signal();

        let first = InnerSubRef::<No>::signal();
        let second = InnerSubRef::<No>::signal();

        sub.add(first.clone());
        sub.add(second.clone());

        let mut o:Option<Mss<No,_>> = Some(o);

        first.addss(self.source.sub_noti(byclone!(first => move |n| {
            match n {
                    Next(v) =>  {
                        let o = o.as_ref().unwrap();
                        o.next(v);
                        if o._is_closed() { first.unsub(); return IsClosed::True; }
                    },
                    Err(e) =>  {
                        first.unsub();
                        o.as_ref().unwrap().err(e);
                    },
                    Comp => {
                        first.unsub();
                        if second.disposed() { return IsClosed::True; }

                        let o:Mss<No,_> = o.take().unwrap();
                        second.add(next.sub_noti(byclone!(second => move |n| {
                        match n {
                            Next(v) => {
                                o.next(v);
                                if o._is_closed() { second.unsub();return IsClosed::True; }
                            },
                            Err(e) => {
                                second.unsub();
                                o.err(e);
                            },
                            Comp => {
                                second.unsub();
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

        sub.into_subref()
    }
}

impl<'a, V:'static+Send+Sync, Src, Next> Observable<'a, V, No, No> for ConcatOp<'a, V, Src, Next, (_No, _No, _No, Yes)> where
    Src : Observable<'a, V, No, No>,
    Next: Observable<'a,V, No, Yes>+'a+Clone,
{
    fn sub(&self, o: Mss<No, impl Observer<V> +'a>) -> SubRef<No>
    {
        let next = self.next.clone();
        let sub = InnerSubRef::<No>::signal();

        let first = InnerSubRef::<No>::signal();
        let second = InnerSubRef::<Yes>::signal();

        sub.add(first.clone());
        sub.addss(second.clone());

        let mut o:Option<Mss<No,_>> = Some(o);

        first.add(self.source.sub_noti(byclone!(first => move |n| {
            match n {
                    Next(v) =>  {
                        let o = o.as_ref().unwrap();
                        o.next(v);
                        if o._is_closed() { first.unsub(); return IsClosed::True; }
                    },
                    Err(e) =>  {
                        first.unsub();
                        o.as_ref().unwrap().err(e);
                    },
                    Comp => {
                        first.unsub();
                        if second.disposed() { return IsClosed::True; }

                        let o:Mss<No,_> = o.take().unwrap();
                        second.add(next.sub_noti(byclone!(second => move |n| {
                        match n {
                            Next(v) => {
                                o.next(v);
                                if o._is_closed() { second.unsub();return IsClosed::True; }
                            },
                            Err(e) => {
                                second.unsub();
                                o.err(e);
                            },
                            Comp => {
                                second.unsub();
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

        sub.into_subref()
    }
}

impl<'a, V:'static+Send+Sync, Src, Next> Observable<'a, V, Yes, No> for ConcatOp<'a, V, Src, Next, (Yes, Yes, _No, _No)> where
    Src : Observable<'a, V, Yes, Yes>,
    Next: Observable<'a,V, No, No>+Send+Sync+'a+Clone,
{
    fn sub(&self, o: Mss<Yes, impl Observer<V> +'a>) -> SubRef<No>
    {
        let next = self.next.clone();
        let sub = InnerSubRef::<Yes>::signal();

        let first = InnerSubRef::<Yes>::signal();

        sub.add(first.clone().into_subref());

        let mut o:Option<Mss<Yes,_>> = Some(o);

        first.add(self.source.sub_noti(byclone!(first, sub => move |n| {
            match n {
                    Next(v) =>  {
                        let o = o.as_ref().unwrap();
                        o.next(v);
                        if o._is_closed() { first.unsub(); return IsClosed::True; }
                    },
                    Err(e) =>  {
                        first.unsub();
                        o.as_ref().unwrap().err(e);
                    },
                    Comp => {
                        first.unsub();
                        if sub.disposed() { return IsClosed::True; }
                        let second = InnerSubRef::<No>::signal();

                        let o:Mss<Yes,_> = o.take().unwrap();
                        second.add(next.sub_noti(byclone!(second => move |n| {
                        match n {
                            Next(v) => {
                                o.next(v);
                                if o._is_closed() { second.unsub();return IsClosed::True; }
                            },
                            Err(e) => {
                                second.unsub();
                                o.err(e);
                            },
                            Comp => {
                                second.unsub();
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

    sub.into_subref()
}
}

impl<'a, V:'static+Send+Sync, Src, Next> Observable<'a, V, Yes, No> for ConcatOp<'a, V, Src, Next, (Yes, No)> where
    Src : Observable<'a, V, Yes, No>,
    Next: Observable<'a,V, No, Yes>+Send+Sync+'a+Clone,
{
    fn sub(&self, o: Mss<Yes, impl Observer<V> +'a>) -> SubRef<No>
    {
        let next = self.next.clone();
        let sub = InnerSubRef::<No>::signal();
        let inner = InnerSubRef::<Yes>::signal().added(get_sync_context().unwrap().create_send(box byclone!(sub => move ||{
            sub.unsub();
        })));
        sub.add(inner.clone());

        let mut o:Option<Mss<Yes,_>> = Some(o);

        sub.add(self.source.sub_noti(byclone!(inner => move |n| {
            match n {
                    Next(v) =>  {
                        let o = o.as_ref().unwrap();
                        o.next(v);
                        if o._is_closed() { inner.unsub(); return IsClosed::True; }
                    },
                    Err(e) =>  {
                        inner.unsub();
                        o.as_ref().unwrap().err(e);
                    },
                    Comp => {
                        let o:Mss<Yes,_> = o.take().unwrap();
                        inner.add(next.sub_noti(byclone!(inner => move |n| {
                        match n {
                            Next(v) => {
                                o.next(v);
                                if o._is_closed() { inner.unsub();return IsClosed::True; }
                            },
                            Err(e) => {
                                inner.unsub();
                                o.err(e);
                            },
                            Comp => {
                                inner.unsub();
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

        sub.into_subref()
    }
}

impl<'a, V:'static+Send+Sync, Src, Next> Observable<'a, V, Yes, No> for ConcatOp<'a, V, Src, Next, (Yes, _No, _No, _No)> where
    Src : Observable<'a, V, Yes, No>,
    Next: Observable<'a,V, No, No>+Send+Sync+'a+Clone,
{
    fn sub(&self, o: Mss<Yes, impl Observer<V> +'a>) -> SubRef<No>
    {
        let next = self.next.clone();
        let sub = InnerSubRef::<No>::signal();
        let first = InnerSubRef::<Yes>::signal().added(get_sync_context().unwrap().create_send(box byclone!(sub => move ||{
            sub.unsub();
        })));

        sub.add(first.clone());

        let mut o:Option<Mss<Yes,_>> = Some(o);

        sub.add(self.source.sub_noti(byclone!(first => move |n| {
            match n {
                    Next(v) =>  {
                        let o = o.as_ref().unwrap();
                        o.next(v);
                        if o._is_closed() { first.unsub(); return IsClosed::True; }
                    },
                    Err(e) =>  {
                        first.unsub();
                        o.as_ref().unwrap().err(e);
                    },
                    Comp => {

                        if first.disposed() { return IsClosed::True; }

                        let o:Mss<Yes,_> = o.take().unwrap();
                        let second = InnerSubRef::<No>::signal();
                        first.add(get_sync_context().unwrap().create_send(box byclone!(second => move ||{
                            second.unsub();
                        })));
                        second.add(first.clone());

                        second.add(next.sub_noti(byclone!(second => move |n| {
                        match n {
                            Next(v) => {
                                o.next(v);
                                if o._is_closed() { second.unsub();return IsClosed::True; }
                            },
                            Err(e) => {
                                second.unsub();
                                o.err(e);
                            },
                            Comp => {
                                second.unsub();
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

        sub.into_subref()
    }
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

        fac::empty::<i32>().concat(fac::empty::<i32>()).concat(fac::just(222)).subf((
            (),
            (),
            || println!("comp")
        ));
    }
}