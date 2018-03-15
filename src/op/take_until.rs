use std::rc::Rc;
use std::any::Any;
use subscriber::*;
use observable::*;
use subref::SubRef;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::marker::PhantomData;
use observable::RxNoti::*;
use observable::*;
use op::*;

pub struct TakeUntilOp<VNoti, Src, Noti>
{
    source : Src,
    noti: Noti,
    PhantomData: PhantomData<VNoti>
}

pub trait ObservableTakeUntil<'a, V, Src, VNoti, Noti> where
    Noti: Observable<'a, VNoti>+Send+Sync,
    Src : Observable<'a, V>,
{
    fn take_until(self, noti:  Noti) -> TakeUntilOp<VNoti, Src, Noti>;
}

impl<'a, V, Src, VNoti, Noti> ObservableTakeUntil<'a, V, Src, VNoti, Noti> for Src where
    Noti: Observable<'a, VNoti>+Send+Sync,
    Src : Observable<'a, V>,
{
    fn take_until(self, noti: Noti) -> TakeUntilOp<VNoti, Src, Noti>
    {
        TakeUntilOp{ source: self, noti: noti, PhantomData }
    }
}

impl<'a, V:'static+Send+Sync, Src, VNoti:'a, Noti> Observable<'a, V> for TakeUntilOp<VNoti, Src, Noti> where
    Noti: Observable<'a, VNoti>+Send+Sync+'a,
    Src : Observable<'a, V>,
{
    #[inline(never)]
    fn sub(&self, dest: impl Observer<V> + Send + Sync+'a) -> SubRef
    {
        let dest = Arc::new(dest);
        let dest2 = dest.clone();

        let sub = SubRef::signal();
        let (sub2 , sub3) = (sub.clone(), sub.clone());

        let sub_noti = self.noti.rx().take(1).sub_noti(move |n|{
            dest.complete();
            sub.unsub();
            IsClosed::True
        });

        if sub_noti.disposed() {
            return SubRef::empty();
        }

        sub2.add(sub_noti);

        let sub = self.source.sub(dest2);
        sub.add(sub2);
        sub3.add(sub);

        sub3
    }
}


#[cfg(test)]
mod test
{
    use super::*;
    use subject::*;
    use fac::*;
    use std::sync::atomic::AtomicIsize;
    use observable::*;
    use scheduler::NewThreadScheduler;

    #[test]
    fn basic()
    {
        let r = AtomicIsize::new(0);
        let subj = Subject::<isize>::new();

         let it = subj.rx().subf(|v| {r.fetch_add(v, Ordering::SeqCst);});
         subj.next(1);

        assert_eq!(r.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn threads()
    {
        let noti = Arc::new(Subject::<i32>::new());
        let subj = Subject::<i32>::new();

        let noti2 = noti.clone();

        let r = Arc::new(AtomicIsize::new(0));
        let r2 = r.clone();

        {
            let it = subj.rx().take_until(noti.clone()).subf(move |v| { r.fetch_add(1, Ordering::SeqCst);});
            subj.next(1);

            let hr = ::std::thread::spawn(move || noti2.next(1));
            hr.join();

            subj.next(1);
        }

        assert_eq!(r2.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn timer()
    {
        let result = Arc::new(AtomicIsize::new(0));
        let (r1, r2) = (result.clone(), result.clone());

        let subj = Subject::new();
        subj.rx().take_until(rxfac::timer(100, None, NewThreadScheduler::get()))
            .subf((move |v| {r1.store(v, Ordering::SeqCst);},
                   (),
                   move || {r2.store(100, Ordering::SeqCst);}
            ));
        subj.next(1);
        assert_eq!(result.load(Ordering::SeqCst), 1);

        ::std::thread::sleep(::std::time::Duration::from_secs(1));
        subj.next(2);

        assert_eq!(result.load(Ordering::SeqCst), 100);
    }
}