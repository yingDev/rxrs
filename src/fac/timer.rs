use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use observable::*;
use unsub_ref::*;
use fac::*;
use scheduler::Scheduler;

pub fn timer(delay: u64, period: Option<u64>, scheduler: Arc<impl Scheduler+Send+Sync+'static>) -> impl Clone+Observable<'static,usize>
{
    timer_dur(Duration::from_millis(delay), period.map(|v| Duration::from_millis(v)), scheduler)
}

pub fn timer_dur(delay: Duration, period: Option<Duration>, scheduler: Arc<impl Scheduler+Send+Sync+'static>) -> impl Clone+Observable<'static,usize>
{
    rxfac::create(move |o|
    {
        let count = AtomicUsize::new(0);
        let scheduler2 = scheduler.clone();
        let sig = UnsubRef::signal();

        let sig2 = sig.clone();

        scheduler.schedule_after(delay, move ||
        {
            if o._is_closed() || sig.disposed() { return sig; }
            o.next(count.fetch_add(1, Ordering::SeqCst));
            if period.is_none() { return sig; }

            scheduler2.schedule_periodic(period.unwrap(), sig, move ||
            {
                if o._is_closed() || sig2.disposed()
                {
                    sig2.unsub();
                    return;
                }
                o.next(count.fetch_add(1, Ordering::SeqCst));
            })
        })

    })
}

pub fn timer_once(delay: u64, scheduler: Arc<impl Scheduler+Send+Sync+'static>) -> impl Observable<'static, usize>
{
    timer(delay, None, scheduler)
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
        timer(100, Some(100), Arc::new(ImmediateScheduler::new())).take(10).subn(|v|  println!("{}", v));

        timer_once(100, Arc::new(ImmediateScheduler::new())).subn(|v| println!("once..."));
    }

    #[test]
    fn new_thread()
    {
        use ::std::sync::Condvar;
        use ::std::sync::Mutex;

        let pair = Arc::new((Mutex::new(false), Condvar::new()));
        let pair2 = pair.clone();

        timer(100, Some(100), NewThreadScheduler::get()).take(10).subf(|v|  println!("{}", v), (), move || {
            let &(ref lock, ref cvar) = &*pair2;
            *lock.lock().unwrap() = true;
                cvar.notify_one();
        });

        let &(ref lock, ref cvar) = &*pair;
        cvar.wait(lock.lock().unwrap());
    }
}
