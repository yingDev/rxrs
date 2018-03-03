use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use observable::*;
use unsub_ref::*;
use fac::*;
use scheduler::Scheduler;

fn timer(delay: Duration, period: Duration, scheduler: Arc<impl Scheduler>) -> impl Observable<usize>
{
    fn recurse(dur: Duration, count: AtomicUsize, scheduler: Arc<impl Scheduler>, dest: Arc<Observer<usize>>, sig: UnsubRef<'static>)
    {
        let scheduler2 = scheduler.clone();

        if dest._is_closed() || sig.disposed() { return; }

        dest.next(count.fetch_add(1, Ordering::SeqCst));

        if dest._is_closed() || sig.disposed() { return; }

        scheduler.schedule_after(dur, move ||{
            recurse(dur, count, scheduler2, dest, sig);
        });
    }

    rxfac::create(move |o|{
        let count = AtomicUsize::new(0);
        let o2 = o.clone();
        let scheduler2 = scheduler.clone();
        let sig = UnsubRef::signal();
        let sig2 = sig.clone();

        scheduler.schedule_after(delay, move || {

            if o2._is_closed() || sig2.disposed() { return; }

            o2.next(count.fetch_add(1, Ordering::SeqCst));
            recurse(period, count, scheduler2, o2, sig2);
        });

        sig
    })
}


#[cfg(test)]
mod test
{
    use super::*;
    use scheduler::ImScheduler;
    use observable::*;
    use op::*;

    #[test]
    fn timer_basic()
    {
        timer(Duration::from_millis(100), Duration::from_millis(100), Arc::new(ImScheduler{})).take(10).subn(|v|  println!("{}", v));
    }
}
