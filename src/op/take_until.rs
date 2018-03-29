use std::rc::Rc;
use observable::*;
use subref::SubRef;
use std::marker::PhantomData;
use util::mss::*;
use std::sync::Mutex;
use std::sync::Arc;

pub struct TakeUntilOp<'b, VNoti, Src:'b, Noti:'b, NotiSSO:?Sized+'static, SSO:?Sized+'static>
{
    source : Src,
    noti: Noti,
    PhantomData: PhantomData<(&'b(), *const VNoti, *const NotiSSO, *const SSO)>
}

pub trait ObservableTakeUntil<'a:'b, 'b, V, Src, VNoti, Noti, NotiSSO:?Sized+'static, SSO:?Sized+'static> where
    Src : Observable<'a, V, SSO>+'b,
    Noti:  Observable<'a, VNoti, NotiSSO>+'b
{
    fn take_until(self, noti: Noti) -> TakeUntilOp<'b, VNoti, Src, Noti, NotiSSO, SSO>;
}

impl<'a:'b,'b, V, Src, VNoti:'a, Noti, NotiSSO:?Sized+'static, SSO:?Sized+'static> ObservableTakeUntil<'a, 'b, V, Src, VNoti, Noti, NotiSSO, SSO>
for Src where
    Src : Observable<'a, V, SSO>+'b,
    Noti: Observable<'a, VNoti, NotiSSO>+'b
{
    fn take_until(self, noti: Noti) -> TakeUntilOp<'b, VNoti, Src, Noti, NotiSSO, SSO>
    {
        TakeUntilOp{ source: self, noti, PhantomData }
    }
}

impl<'a:'b, 'b, V:'a+Send+Sync, Src, VNoti:'a, Noti> Observable<'a, V, Yes>
for TakeUntilOp<'b, VNoti, Src, Noti, Yes, Yes> where
    Src : Observable<'a, V, Yes>+'b,
    Noti: Observable<'a, VNoti, Yes>+'b
{
    fn sub(&self, o: Mss<Yes, impl Observer<V> +'a>) -> SubRef
    {
        use observable::RxNoti::*;

        let o: Arc<Mss<Yes, _>> = Arc::new(o);
        let sub = SubRef::signal();
        let gate = Arc::new(Mutex::new(()));

        sub.add(self.noti.rx().sub_noti(byclone!(o,sub,gate => move |n|{
            let lock = gate.lock().unwrap();
            if !sub.disposed(){
                o.complete();
                sub.unsub();
            }
            IsClosed::True
        })).added(sub.clone()));

        if sub.disposed() {
            return sub;
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
        })).added(sub.clone()));

        sub
    }
}

impl<'a:'b, 'b, V:'a, Src, VNoti:'a, Noti> Observable<'a, V, No>
for TakeUntilOp<'b, VNoti, Src, Noti, No, No> where
    Src : Observable<'a, V, No>+'b,
    Noti: Observable<'a, VNoti, No>+'b
{
    fn sub(&self, o: Mss<No, impl Observer<V> +'a>) -> SubRef
    {
        use observable::RxNoti::*;

        let sub = SubRef::signal();

        let o = Rc::new(o);
        let notisub = self.noti.rx().sub_noti(byclone!(o,sub => move |n|{
           // if !sub.disposed(){
                o.complete();
                sub.unsub();
           // }
            IsClosed::True
        }));
        sub.add(notisub.added(sub.clone()));

        if sub.disposed() {
            return sub;
        }

        sub.add(self.source.sub_noti(byclone!(o, sub => move |n| {
            if sub.disposed() || o._is_closed() { //todo: other operators too...
                sub.unsub();
                return IsClosed::True;
            }
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
        })).added(sub.clone()));

        sub
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    use fac;
    use scheduler::NewThreadScheduler;
    use std::thread;
    use std::time::Duration;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn basic_ss()
    {
        let sig = fac::timer_ss(200, None, NewThreadScheduler::get());

        let seq = fac::create_sso(|o| {
            let stop = SubRef::signal();
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


}