use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use observable::*;
use subref::*;
use fac::*;
use scheduler::Scheduler;
use util::mss::*;
use std::marker::PhantomData;
use scheduler::SchedulerPeriodic;

pub fn timer(delay: u64, period: impl Into<Option<u64>>, scheduler: Arc<impl SchedulerPeriodic<SSA=No>+Send+Sync+'static>) -> impl Observable<'static, usize, No+'static>
{
    timer_dur(Duration::from_millis(delay), period.into().map(|v| Duration::from_millis(v)), scheduler)
}

pub fn timer_ss(delay: u64, period: impl Into<Option<u64>>, scheduler: Arc<impl SchedulerPeriodic<SSA=Yes>+Send+Sync+'static>) -> impl Observable<'static, usize, Yes>
{
    timer_dur_ss(Duration::from_millis(delay), period.into().map(|v| Duration::from_millis(v)), scheduler)
}

macro_rules! fnbody(($s:ty, $delay:ident, $sch:ident, $period:ident, $o:ident) => {{
    let scheduler2 = $sch.clone();
    let sig = SubRef::signal();

    $sch.schedule_after($delay, Mss::<$s,_>::new(move ||
    {
        if $o._is_closed() || sig.disposed() { return sig.clone(); }

        let count = AtomicUsize::new(0);
        $o.next(count.fetch_add(1, Ordering::SeqCst));
        if $period.is_none() { return sig.clone(); }

        scheduler2.schedule_periodic($period.unwrap(), sig.clone(), Mss::<$s,_>::new(move ||
        {
            if $o._is_closed() || sig.disposed()
            {
                sig.unsub();
                return;
            }
            $o.next(count.fetch_add(1, Ordering::SeqCst));
        }))
    }))
}});

pub fn timer_dur(delay: Duration, period: impl Into<Option<Duration>>, scheduler: Arc<impl SchedulerPeriodic<SSA=No>+Send+Sync+'static>) -> impl Observable<'static, usize, No+'static>
{
    let period = period.into();
    create_boxed(move |o: Mss<No, Box<Observer<usize>+'static>>| {
        fnbody!(No, delay, scheduler, period, o)
    })
}

pub fn timer_dur_ss(delay: Duration, period: impl Into<Option<Duration>>, scheduler: Arc<impl SchedulerPeriodic<SSA=Yes>+Send+Sync+'static>) -> impl Observable<'static, usize, Yes>
{
    let period = period.into();
    create_sso(move |o: Mss<Yes, Box<Observer<usize>+'static>>| {
        fnbody!(Yes, delay, scheduler, period, o)
    })
}

#[cfg(test)]
mod test
{
    use super::*;
    use scheduler::*;
    use observable::*;
    use op::*;

    #[test]
    fn timer_basic()
    {

    }

    #[test]
    fn new_thread()
    {
//        use ::std::sync::Condvar;
//        use ::std::sync::Mutex;
//
//        let pair = Arc::new((Mutex::new(false), Condvar::new()));
//        let pair2 = pair.clone();
//
//        timer(100, 100, NewThreadScheduler::get()).take(10).subf((|v|  println!("{}", v), (), move || {
//            let &(ref lock, ref cvar) = &*pair2;
//            *lock.lock().unwrap() = true;
//                cvar.notify_one();
//        }));
//
//        let &(ref lock, ref cvar) = &*pair;
//        cvar.wait(lock.lock().unwrap());
    }
}
