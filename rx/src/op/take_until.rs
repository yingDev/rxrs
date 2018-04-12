use std::rc::Rc;
use observable::*;
use subref::*;
use std::marker::PhantomData;
use util::mss::*;
use std::sync::Mutex;
use std::sync::Arc;
use observable::RxNoti::*;
use scheduler::get_sync_context;

//todo

pub struct TakeUntilOp<'b, VNoti, V, Src:'b, Noti:'b, SSFlags:Sized>
{
    source : Src,
    noti: Noti,
    PhantomData: PhantomData<(&'b(), *const VNoti, *const V, *const SSFlags)>
}

pub trait ObservableTakeUntil<'a:'b, 'b, V, Src, VNoti, Noti, SSFlags>
{
    fn take_until(self, noti: Noti) -> TakeUntilOp<'b, VNoti, V, Src, Noti, SSFlags>;
}

impl<'a:'b,'b, V, Src, VNoti:'a, Noti, SSFlags> ObservableTakeUntil<'a, 'b, V, Src, VNoti, Noti, SSFlags> for Src
{
    fn take_until(self, noti: Noti) -> TakeUntilOp<'b, VNoti, V, Src, Noti, SSFlags>
    {
        TakeUntilOp{ source: self, noti, PhantomData }
    }
}

impl<'a:'b, 'b, V:'a+Send+Sync, Src, VNoti:'a, Noti> Observable<'a, V, Yes, Yes> for TakeUntilOp<'b, VNoti, V, Src, Noti, (Yes, Yes, Yes, Yes)> where
    Src : Observable<'a, V, Yes, Yes>+'b,
    Noti: Observable<'a, VNoti, Yes, Yes>+'b
{
    fn sub(&self, o: Mss<Yes, impl Observer<V> +'a>) -> SubRef<Yes>
    {
        let o: Arc<Mss<Yes, _>> = Arc::new(o);
        let sub = InnerSubRef::<Yes>::signal();
        let gate = Arc::new(Mutex::new(()));

        sub.add(self.noti.rx().sub_noti(byclone!(o,sub,gate => move |n|{
            let lock = gate.lock().unwrap();
            if !sub.disposed(){
                sub.unsub();
                o.complete();
            }
            IsClosed::True
        })));

        if sub.disposed() {
            return sub.into_subref();
        }

        sub.add(self.source.sub_noti(byclone!(o, sub, gate => move |n| {
            let lock = gate.lock().unwrap();
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

          return IsClosed::Default;
        })));

        sub.into_subref()
    }
}

impl<'a:'b, 'b, V:'a+Send+Sync, Src, VNoti:'a, Noti> Observable<'a, V, Yes, Yes> for TakeUntilOp<'b, VNoti, V, Src, Noti, (Yes, _No, Yes, Yes)> where
    Src : Observable<'a, V, Yes, Yes>+'b,
    Noti: Observable<'a, VNoti, No, Yes>+'b
{
    fn sub(&self, o: Mss<Yes, impl Observer<V> +'a>) -> SubRef<Yes>
    {
        let o: Arc<Mss<Yes, _>> = Arc::new(o);
        let sub = InnerSubRef::<Yes>::signal();
        let gate = Arc::new(Mutex::new(()));

        sub.add(self.noti.rx().sub_noti(byclone!(o,sub,gate => move |n|{
            let lock = gate.lock().unwrap();
            if !sub.disposed(){
                sub.unsub();
                o.complete();
            }
            IsClosed::True
        })));

        if sub.disposed() {
            return sub.into_subref();
        }

        sub.add(self.source.sub_noti(byclone!(o, sub, gate => move |n| {
            let lock = gate.lock().unwrap();
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

          return IsClosed::Default;
        })));

        sub.into_subref()
    }
}


impl<'a:'static, V:'a+Send+Sync, Src, VNoti:'a, Noti> Observable<'a, V, No, No> for TakeUntilOp<'static, VNoti, V, Src, Noti, (_No, Yes, Yes, Yes)> where
    Src : Observable<'a, V, No, No>+'static,
    Noti: Observable<'a, VNoti, Yes, Yes>+'static
{
    fn sub(&self, o: Mss<No, impl Observer<V> +'a>) -> SubRef<No>
    {
        let o = Rc::new(o);

        let sub = InnerSubRef::<No>::signal();
        let notisub = InnerSubRef::<Yes>::signal();

        sub.add(notisub.clone());

        let inner = get_sync_context().unwrap().create_send(box byclone!(sub, o => move ||{
            if !sub.disposed() {
                sub.unsub();
                o.complete();
            }
        }));
        sub.add(inner.clone());

        notisub.add(self.noti.rx().sub_noti(byclone!(notisub,inner => move |n|{
            notisub.unsub();
            inner.unsub();
            IsClosed::True
        })));

        if sub.disposed() {
            return sub.into_subref();
        }

        sub.add(self.source.sub_noti(byclone!(o, sub => move |n| {
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

          return IsClosed::Default;
        })));

        sub.into_subref()
    }
}

macro_rules! fn_no_xx_sub {
($sss:ty) => {
    fn sub(&self, o: Mss<No, impl Observer<V> +'a>) -> SubRef<$sss>
    {
        let sub = InnerSubRef::<$sss>::signal();

        let o = Rc::new(o);
        sub.add(self.noti.rx().sub_noti(byclone!(o,sub => move |n|{
            sub.unsub();
            o.complete();
            IsClosed::True
        })));

        if sub.disposed() {
            return sub.into_subref();
        }

        sub.add(self.source.sub_noti(byclone!(o, sub => move |n| {
            match n {
                Next(v) => {
                    o.next(v);
                    if o._is_closed() {
                        sub.unsub();
                        return IsClosed::True;
                    }
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

          return IsClosed::Default;
        })));

        sub.into_subref()
    }
};
}

macro_rules! impl_no_xx {
($srcsss:ty, $notisss:ty => $outsss:ty) => {
    impl<'a:'b, 'b, V:'a, Src, VNoti:'a, Noti> Observable<'a, V, No, $outsss> for TakeUntilOp<'b, VNoti, V, Src, Noti, (_No, _No, *const $srcsss, *const $notisss)> where
        Src : Observable<'a, V, No, $srcsss>+'b,
        Noti: Observable<'a, VNoti, No, $notisss>+'b
        {
            fn_no_xx_sub!($outsss);
        }
    };
}

impl_no_xx!(No, No => No);
impl_no_xx!(No, Yes => No);
impl_no_xx!(Yes, No => No);
impl_no_xx!(Yes, Yes => Yes);

#[cfg(test)]
mod test
{
    use super::*;
    use fac;
    use test_fixture::*;
    use scheduler::NewThreadScheduler;
    use std::thread;
    use std::time::Duration;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn basic_ss()
    {
        let sig = fac::timer_ss(200, None, NewThreadScheduler::get());

        let seq = fac::create_sso(|o| {
            let stop = InnerSubRef::<Yes>::signal();
            thread::spawn(byclone!(stop => move || {
                for i in 0..10 {
                    if stop.disposed() {
                        return;
                    }
                    o.next(i);
                    thread::sleep(Duration::from_millis(100));
                }
                if stop.disposed() {
                    return;
                }

                o.complete();
                stop.unsub();
            }));
            stop
        });

        let out = Arc::new(AtomicUsize::new(0));
        seq.take_until(sig).subf(byclone!(out => move |v| { out.fetch_add(1, Ordering::SeqCst); } ));

        thread::sleep(Duration::from_millis(1000));
        assert_eq!(out.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn empty_sig()
    {
        let src = SimpleObservable;
        let mut done = false;

        src.take_until(fac::empty::<()>())
            .subf((
                |v| panic!("should not get called"),
                (),
                || done = true
        ));

        assert_eq!(done, true);
    }

}