use std::time::Duration;
use subref::SubRef;
use std::sync::Arc;
use std::sync::Once;
use std::sync::ONCE_INIT;
use util::mss::*;
use std::marker::PhantomData;
use std::sync::Condvar;
use std::sync::Mutex;
use std::collections::VecDeque;
use std::sync::atomic::*;
use observable::*;
use observable::RxNoti::*;
use util::AtomicOption;
use std::any::Any;
use util::mss::*;

//todo: facade

pub trait Scheduler
{
    type SSA:?Sized;

    fn schedule(&self, act: Mss<Self::SSA,impl 'static+FnOnce()->SubRef>) -> SubRef;
    fn schedule_after(&self, due: Duration, act: Mss<Self::SSA, impl 'static+FnOnce()->SubRef>) -> SubRef;
}

pub trait SchedulerObserveOn<'sa, V:'static+Send+Sync, Src, SrcSSO:?Sized, ObserveOn: Observable<'static, V, Self::SSA>> : Scheduler where Src: Observable<'sa, V, SrcSSO>
{
    fn observe_on(&self, source: Src) -> ObserveOn;
}

pub trait SchedulerPeriodic : Scheduler
{
    fn schedule_periodic(&self, period: Duration,sigStop: SubRef, act: Mss<Self::SSA, impl 'static+Fn()>) -> SubRef;
}

pub trait SchedulerLongRunning : Scheduler
{
    fn schedule_long_running(&self, sigStop: SubRef, act: Mss<Self::SSA, impl 'static+FnOnce()>) -> SubRef;
}

pub struct ImmediateScheduler;

impl ImmediateScheduler
{
    pub fn new() -> ImmediateScheduler { ImmediateScheduler }
}

impl Scheduler for ImmediateScheduler
{
    type SSA = No;

    fn schedule(&self, act: Mss<Self::SSA,impl 'static+FnOnce()->SubRef>) -> SubRef
    {
        (act.into_inner())()
    }

    fn schedule_after(&self, due: Duration, act: Mss<Self::SSA,impl 'static+FnOnce()->SubRef>) -> SubRef
    {
        ::std::thread::sleep(due);
        (act.into_inner())()
    }
}

impl SchedulerPeriodic for ImmediateScheduler
{
    fn schedule_periodic(&self, period: Duration, sigStop: SubRef, act: Mss<Self::SSA,impl 'static+Fn()>) -> SubRef
    {
        while ! sigStop.disposed()
            {
                ::std::thread::sleep(period);
                if sigStop.disposed() {
                    break;
                }
                act();
            }

        sigStop
    }
}

impl SchedulerLongRunning for ImmediateScheduler
{
    fn schedule_long_running(&self, sigStop: SubRef, act: Mss<Self::SSA, impl 'static+FnOnce()>) -> SubRef
    {
        if sigStop.disposed() { return sigStop; }
        let act = act.into_inner();
        self.schedule(Mss::no(||{
            if sigStop.disposed() { return sigStop; }
            act();
            sigStop
        }))
    }
}




static mut NEW_THREAD_SCHEDULER: Option<Arc<NewThreadScheduler>> = None;
static NEW_THREAD_SCHEDULER_INIT: Once = ONCE_INIT;

pub struct NewThreadScheduler;
impl NewThreadScheduler
{
    pub fn get() -> Arc<NewThreadScheduler> {
        NEW_THREAD_SCHEDULER_INIT.call_once(|| {
            unsafe { NEW_THREAD_SCHEDULER = Some(Arc::new(NewThreadScheduler{})); }
        });
        unsafe { NEW_THREAD_SCHEDULER.as_ref().unwrap().clone() }
    }
}

impl Scheduler for NewThreadScheduler
{
    type SSA = Yes;

    fn schedule(&self, act: Mss<Self::SSA,impl 'static+FnOnce()->SubRef>) -> SubRef
    {
        let unsub = SubRef::signal();
        let unsub2 = unsub.clone();

       ::std::thread::spawn(move ||{
           unsub2.add((act.into_inner())());
       });

       unsub
    }

    fn schedule_after(&self, due: Duration, act: Mss<Self::SSA,impl 'static+FnOnce()->SubRef>) -> SubRef
    {
        let unsub = SubRef::signal();
        let unsub2 = unsub.clone();

        ::std::thread::spawn(move ||{
            ::std::thread::sleep(due);
            if ! unsub2.disposed() { unsub2.add((act.into_inner())()); }
        });

        unsub
    }
}

impl SchedulerLongRunning for NewThreadScheduler
{
    fn schedule_long_running(&self, sigStop: SubRef, act: Mss<Self::SSA, impl 'static+FnOnce()>) -> SubRef
    {
        ::std::thread::spawn(move || {
            (act.into_inner())();
        });
        sigStop
    }
}

impl SchedulerPeriodic for NewThreadScheduler
{
    fn schedule_periodic(&self, period: Duration, sigStop: SubRef, act: Mss<Self::SSA,impl 'static+Fn()>) -> SubRef
    {
        let stop = sigStop.clone();
        ::std::thread::spawn(move || {
                while ! stop.disposed(){
                    ::std::thread::sleep(period);
                    if stop.disposed() { break; }
                    act();
                }
            });
        sigStop
    }
}

impl<'sa, V:'static+Send+Sync, Src> SchedulerObserveOn<'sa, V, Src, Yes, ObserveOnNewThread<'sa, V, Src, Yes>> for NewThreadScheduler  where Src: Observable<'sa, V, Yes>
{
    fn observe_on(&self, source: Src) -> ObserveOnNewThread<'sa, V, Src, Yes>
    {
        ObserveOnNewThread { source, scheduler: Self::get(), PhantomData }
    }
}


impl<'sa, V:'static+Send+Sync, Src> SchedulerObserveOn<'sa, V, Src, No, ObserveOnNewThread<'sa, V, Src, No>> for NewThreadScheduler  where Src: Observable<'sa, V, No>
{
    fn observe_on(&self, source: Src) -> ObserveOnNewThread<'sa, V, Src, No>
    {
        ObserveOnNewThread { source, scheduler: Self::get(), PhantomData }
    }
}


pub struct ObserveOnNewThread<'sa, V:'static, Src, SrcSSO:?Sized> where Src: Observable<'sa, V, SrcSSO>
{
    source: Src,
    scheduler: Arc<NewThreadScheduler>,
    PhantomData: PhantomData<(&'sa (), V, SrcSSO)>
}

macro_rules! fn_sub(
($s: ty)=>{
    fn sub(&self, o: Mss<Yes, impl Observer<V> +'static>) -> SubRef
    {
        let scheduler = self.scheduler.clone();
        let queue =  Arc::new((Condvar::new(), Mutex::new(VecDeque::new()), AtomicOption::new()));
        let stopped = Arc::new(AtomicBool::new(false));

        let q = queue.clone();
        let sub = SubRef::signal();
        let sub2 = sub.clone();

        let stopped2 = stopped.clone();
        sub.add(self.scheduler.schedule_long_running(sub.clone(), Mss::new(move || {
            dispatch(o, q, stopped2);
        })));

        sub.add(self.source.sub_noti(move |n| {
            let &(ref cond, ref q, ref err) = &*queue;

            match n {
                Next(v) => {
                    q.lock().unwrap().push_back(v);
                },
                Err(e) => {
                    if stopped.compare_and_swap(false, true, Ordering::Acquire) {
                        q.lock().unwrap();
                        sub2.unsub();
                        err.swap(e, Ordering::SeqCst);
                    }
                },
                Comp => {
                    if stopped.compare_and_swap(false, true, Ordering::Acquire) {
                        sub2.unsub();
                    }
                }
            }
            cond.notify_one();
            if stopped.load(Ordering::Acquire) {
                return IsClosed::True;
            }
            return IsClosed::Default;
        }));

        sub
    }
});

impl<'sa, V:'static+Send+Sync, Src> Observable<'static, V, Yes> for ObserveOnNewThread<'sa, V, Src, Yes> where Src: Observable<'sa, V, Yes>
{
    fn_sub!(Yes);
}
impl<'sa, V:'static+Send+Sync, Src> Observable<'static, V, Yes> for ObserveOnNewThread<'sa, V, Src, No> where Src: Observable<'sa, V, No>
{
    fn sub(&self, o: Mss<Yes, impl Observer<V> +'static>) -> SubRef
    {
        let scheduler = self.scheduler.clone();
        let queue =  Arc::new((Condvar::new(), Mutex::new(VecDeque::new()), AtomicOption::new()));
        let stopped = Arc::new(AtomicBool::new(false));

        let q = queue.clone();
        let sub = SubRef::signal();
        let sub2 = sub.clone();

        let stopped2 = stopped.clone();
        sub.add(self.scheduler.schedule_long_running(sub.clone(), Mss::new(move || {
            dispatch(o, q, stopped2);
        })));

        let src_sub = self.source.sub_noti(move |n| {
            let &(ref cond, ref q, ref err) = &*queue;

            match n {
                Next(v) => {
                    q.lock().unwrap().push_back(v);
                },
                Err(e) => {
                    if stopped.compare_and_swap(false, true, Ordering::Acquire) {
                        q.lock().unwrap();
                        sub2.unsub();
                        err.swap(e, Ordering::SeqCst);
                    }
                },
                Comp => {
                    if stopped.compare_and_swap(false, true, Ordering::Acquire) {
                        sub2.unsub();
                    }
                }
            }
            cond.notify_one();
            if stopped.load(Ordering::Acquire) {
                return IsClosed::True;
            }
            return IsClosed::Default;
        });

        sub.add(src_sub);

        sub
    }
}

#[inline(never)]
fn dispatch<V>(o: Mss<Yes, impl Observer<V>+'static>, queue: Arc<(Condvar, Mutex<VecDeque<V>>, AtomicOption<Arc<Any+Send+Sync>>)>, stopped: Arc<AtomicBool>)
{
    loop {
        let &(ref cond, ref lock, ref err) = &*queue;

        //todo: take all at once
        while let Some(v) = lock.lock().unwrap().pop_front() {
            if o._is_closed() { break; }
            o.next(v);
        }

        if stopped.load(Ordering::Acquire) {
            if let Some(e) = err.take(Ordering::Acquire) {
                o.err(e);
            } else {
                o.complete();
            }
            return;
        }

        cond.wait(lock.lock().unwrap());
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    use test_fixture::*;

    #[test]
    fn requirements()
    {
        fn a<'sa, Src, OO:Observable<'static, i32, Yes>>(src:Src, sch: Arc<impl Scheduler<SSA=Yes>+SchedulerLongRunning+SchedulerPeriodic+SchedulerObserveOn<'sa, i32, Src, Yes, OO>>) where Src: Observable<'sa, i32, Yes>
        {
            println!("ok");
        }
        a(ThreadedObservable,  NewThreadScheduler::get());
    }
}