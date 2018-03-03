use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use observable::*;
use unsub_ref::*;
use fac::*;
use scheduler::Scheduler;

fn timer(delay: Duration, period: Duration, scheduler: Arc<impl Scheduler+Send+Sync+'static>) -> impl Observable<usize>
{
    fn recurse(dur: Duration, count: AtomicUsize, scheduler: Arc<impl Scheduler+Send+Sync+'static>, dest: Arc<Observer<usize>+Send+Sync>, sig: UnsubRef<'static>)
    {
        let scheduler2 = scheduler.clone();

        if dest._is_closed() || sig.disposed() { return; }

        dest.next(count.fetch_add(1, Ordering::SeqCst));

        if dest._is_closed() || sig.disposed() { return; }

        scheduler.schedule_after(dur, move ||{
            recurse(dur, count, scheduler2, dest, sig);
            UnsubRef::empty()
        });
    }

    rxfac::create(move |o|{
        let count = AtomicUsize::new(0);
        let o2 = o.clone();
        let scheduler2 = scheduler.clone();
        let sig = UnsubRef::signal();
        let sig2 = sig.clone();
        let sig3 = sig.clone();

        scheduler.schedule_after(delay, move || {

            if o2._is_closed() || sig2.disposed() { return sig2; }

            o2.next(count.fetch_add(1, Ordering::SeqCst));
            recurse(period, count, scheduler2, o2, sig2);
            return sig3;
        })
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
        timer(Duration::from_millis(100), Duration::from_millis(100), Arc::new(ImScheduler::new())).take(10).subn(|v|  println!("{}", v));
    }

    #[test]
    fn new_thread()
    {
        use ::std::sync::Condvar;
        use ::std::sync::Mutex;

        let pair = Arc::new((Mutex::new(false), Condvar::new()));
        let pair2 = pair.clone();

        timer(Duration::from_millis(100), Duration::from_millis(100), Arc::new(NewThreadScheduler::new())).take(10).subf(|v|  println!("{}", v), (), move || {
            let &(ref lock, ref cvar) = &*pair2;
            *lock.lock().unwrap() = true;
                cvar.notify_one();
        });

        let &(ref lock, ref cvar) = &*pair;
        cvar.wait(lock.lock().unwrap());
    }
}
