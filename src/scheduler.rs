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
use std::mem;

//todo: facade
//todo: act -> SubRef seem useless
pub trait Scheduler<SSA:?Sized+'static>
{
    fn schedule(&self, act: Mss<SSA,impl 'static+FnOnce()->SubRef>) -> SubRef;
    fn schedule_after(&self, due: Duration, act: Mss<SSA, impl 'static+FnOnce()->SubRef>) -> SubRef;
}

pub trait SchedulerObserveOn<'sa, V:'static+Send+Sync, Src, SSA:?Sized+'static, SrcSSO:?Sized, ObserveOn: Observable<'static, V, SSA>> : Scheduler<SSA> where Src: Observable<'sa, V, SrcSSO>
{
    fn observe_on(&self, source: Src) -> ObserveOn;
}

pub trait SchedulerPeriodic<SSA:?Sized+'static> : Scheduler<SSA>
{
    fn schedule_periodic(&self, period: Duration,sigStop: SubRef, act: Mss<SSA, impl 'static+Fn()>) -> SubRef;
}

pub trait SchedulerLongRunning<SSA:?Sized+'static> : Scheduler<SSA>
{
    fn schedule_long_running(&self, sigStop: SubRef, act: Mss<SSA, impl 'static+FnOnce()>) -> SubRef;
}

pub struct ImmediateScheduler;

impl ImmediateScheduler
{
    pub fn get() -> Arc<ImmediateScheduler>
    {
        static mut VALUE: Option<Arc<ImmediateScheduler>> = None;
        static VALUE_INIT: Once = ONCE_INIT;
        VALUE_INIT.call_once(|| {
            unsafe { VALUE = Some(Arc::new(ImmediateScheduler{})); }
        });
        unsafe { VALUE.as_ref().unwrap().clone() }
    }
}

impl Scheduler<No> for ImmediateScheduler
{
    fn schedule(&self, act: Mss<No,impl 'static+FnOnce()->SubRef>) -> SubRef
    {
        (act.into_inner())()
    }

    fn schedule_after(&self, due: Duration, act: Mss<No,impl 'static+FnOnce()->SubRef>) -> SubRef
    {
        ::std::thread::sleep(due);
        (act.into_inner())()
    }
}

impl SchedulerPeriodic<No> for ImmediateScheduler
{
    fn schedule_periodic(&self, period: Duration, sigStop: SubRef, act: Mss<No,impl 'static+Fn()>) -> SubRef
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

impl SchedulerLongRunning<No> for ImmediateScheduler
{
    fn schedule_long_running(&self, sigStop: SubRef, act: Mss<No, impl 'static+FnOnce()>) -> SubRef
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




pub struct NewThreadScheduler;
impl NewThreadScheduler
{
    pub fn get() -> Arc<NewThreadScheduler>
    {
        static mut VALUE: Option<Arc<NewThreadScheduler>> = None;
        static VALUE_INIT: Once = ONCE_INIT;
        VALUE_INIT.call_once(|| {
            unsafe { VALUE = Some(Arc::new(NewThreadScheduler{})); }
        });
        unsafe { VALUE.as_ref().unwrap().clone() }
    }
}

impl Scheduler<Yes> for NewThreadScheduler
{
    fn schedule(&self, act: Mss<Yes,impl 'static+FnOnce()->SubRef>) -> SubRef
    {
        let unsub = SubRef::signal();
        let unsub2 = unsub.clone();

       ::std::thread::spawn(move ||{
           unsub2.add((act.into_inner())());
       });

       unsub
    }

    fn schedule_after(&self, due: Duration, act: Mss<Yes,impl 'static+FnOnce()->SubRef>) -> SubRef
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

impl SchedulerLongRunning<Yes> for NewThreadScheduler
{
    fn schedule_long_running(&self, sigStop: SubRef, act: Mss<Yes, impl 'static+FnOnce()>) -> SubRef
    {
        ::std::thread::spawn(move || {
            (act.into_inner())();
        });
        sigStop
    }
}

impl SchedulerPeriodic<Yes> for NewThreadScheduler
{
    fn schedule_periodic(&self, period: Duration, sigStop: SubRef, act: Mss<Yes,impl 'static+Fn()>) -> SubRef
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

impl<'sa, V:'static+Send+Sync, Src> SchedulerObserveOn<'sa, V, Src, Yes, Yes, ObserveOnNewThread<'sa, V, Src, Yes>> for NewThreadScheduler  where Src: Observable<'sa, V, Yes>
{
    fn observe_on(&self, source: Src) -> ObserveOnNewThread<'sa, V, Src, Yes>
    {
        ObserveOnNewThread { source, scheduler: Self::get(), PhantomData }
    }
}


impl<'sa, V:'static+Send+Sync, Src> SchedulerObserveOn<'sa, V, Src, Yes, No, ObserveOnNewThread<'sa, V, Src, No>> for NewThreadScheduler  where Src: Observable<'sa, V, No>
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
        let queue =  Arc::new((Condvar::new(), Mutex::new(VecDeque::new()), AtomicOption::new()));
        let stopped = Arc::new(AtomicBool::new(false));

        let q = queue.clone();
        let sub = SubRef::signal();

        sub.add(self.scheduler.schedule_long_running(sub.clone(), Mss::new(byclone!(q, stopped => move || {
            dispatch(o, q, stopped);
        }))));

        sub.add(self.source.sub_noti(byclone!(sub => move |n| {
            let &(ref cond, ref q, ref err) = &*queue;

            match n {
                Next(v) => {
                    q.lock().unwrap().push_back(v);
                },
                Err(e) => {
                    if stopped.compare_and_swap(false, true, Ordering::Acquire) {
                        //q.lock().unwrap();
                        sub.unsub();
                        err.swap(e, Ordering::SeqCst);
                    }
                },
                Comp => {
                    if stopped.compare_and_swap(false, true, Ordering::Acquire) {
                        sub.unsub();
                    }
                }
            }
            cond.notify_one();
            if stopped.load(Ordering::Acquire) {
                return IsClosed::True;
            }
            return IsClosed::Default;
        })).added(sub.clone()));

        sub
    }
});

impl<'sa, V:'static+Send+Sync, Src> Observable<'static, V, Yes> for ObserveOnNewThread<'sa, V, Src, Yes> where Src: Observable<'sa, V, Yes>
{
    fn_sub!(Yes);
}
impl<'sa, V:'static+Send+Sync, Src> Observable<'static, V, Yes> for ObserveOnNewThread<'sa, V, Src, No> where Src: Observable<'sa, V, No>
{
    fn_sub!(No);
}

#[inline(never)]
fn dispatch<V>(o: Mss<Yes, impl Observer<V>+'static>, queue: Arc<(Condvar, Mutex<VecDeque<V>>, AtomicOption<ArcErr>)>, stopped: Arc<AtomicBool>)
{
    let mut buffer = VecDeque::new();
        let &(ref cond, ref lock, ref err) = &*queue;

    'out: loop {
        let mut lock = lock.lock().unwrap();
        if let Some(v) = lock.pop_front() {
            if o._is_closed() { break; }
            buffer.push_back(v);
        }
        if buffer.len() > 0 {
            mem::drop(lock);
            while let Some(v) = buffer.pop_front() {
                if o._is_closed() { break 'out; }
                o.next(v);
            }
        }else {
            if stopped.load(Ordering::Acquire) {
                mem::drop(lock);
                if let Some(e) = err.take(Ordering::Acquire) {
                    o.err(e);
                } else {
                    o.complete();
                }
                return;
            }
            let r = cond.wait(lock);
            r.ok();
        }
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
        fn a<'sa, Src, OO:Observable<'static, i32, Yes>>(src:Src, sch: Arc<impl SchedulerLongRunning<Yes>+SchedulerPeriodic<Yes>+SchedulerObserveOn<'sa, i32, Src, Yes, Yes, OO>>) where Src: Observable<'sa, i32, Yes>
        {
            println!("ok");
        }
        a(ThreadedObservable,  NewThreadScheduler::get());
    }
}