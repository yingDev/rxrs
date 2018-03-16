use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use observable::*;
use subref::*;
use fac::*;
use scheduler::Scheduler;

pub fn timer(delay: u64, period: Option<u64>, scheduler: Arc<impl Scheduler+Send+Sync+'static>) -> Arc<Observable<'static,usize>+'static+Send+Sync>
{
    timer_dur(Duration::from_millis(delay), period.map(|v| Duration::from_millis(v)), scheduler)
}

pub fn timer_dur(delay: Duration, period: Option<Duration>, scheduler: Arc<impl Scheduler+Send+Sync+'static>) -> Arc<Observable<'static,usize>+'static+Send+Sync>
{
    rxfac::create(move |o|
    {
        let count = AtomicUsize::new(0);
        let scheduler2 = scheduler.clone();
        let sig = SubRef::signal();

        let sig2 = sig.clone();
        let o = o.clone();

        scheduler.schedule_after(delay, move ||
        {
            if o._is_closed() || sig.disposed() { return sig.clone(); }
            o.next(count.fetch_add(1, Ordering::SeqCst));
            if period.is_none() { return sig.clone(); }

            let o2 = o.clone();
            let sig3 = sig2.clone();

            scheduler2.schedule_periodic(period.unwrap(), sig.clone(), move ||
            {
                if o2._is_closed() || sig3.disposed()
                {
                    sig3.unsub();
                    return;
                }
                o2.next(count.fetch_add(1, Ordering::SeqCst));
            })
        })

    })
}

pub fn timer_once(delay: u64, scheduler: Arc<impl Scheduler+Send+Sync+'static>) -> Arc<Observable<'static,usize>+'static+Send+Sync>
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
    use observable;

    #[test]
    fn timer_basic()
    {
        timer(100, Some(100), Arc::new(ImmediateScheduler::new())).take(10).subf(|v|  println!("{}", v));

        timer_once(100, Arc::new(ImmediateScheduler::new())).subf(|v| println!("once..."));
    }

    #[test]
    fn new_thread()
    {
        use ::std::sync::Condvar;
        use ::std::sync::Mutex;

        let pair = Arc::new((Mutex::new(false), Condvar::new()));
        let pair2 = pair.clone();

        timer(100, Some(100), NewThreadScheduler::get()).take(10).subf((|v|  println!("{}", v), (), move || {
            let &(ref lock, ref cvar) = &*pair2;
            *lock.lock().unwrap() = true;
                cvar.notify_one();
        }));

        let &(ref lock, ref cvar) = &*pair;
        cvar.wait(lock.lock().unwrap());
    }

    #[test]
    fn lifetime()
    {
        use op::*;

        let i = 123;
        let x = X{ a: &324234 }.rx();

        x.take(1).subf(|v| println!("ok {}", 2123));
    }

    struct X<'a>
    {
        a: &'a i32
    }

    impl<'a> Observable<'static, i32> for X<'a>
    {
        fn sub(&self, o: Arc<Observer<i32>+'static+Send+Sync>) -> SubRef
        {
            ::std::thread::spawn(move || {
                o.next(123);
                o.complete();
            });


            SubRef::empty()
        }
    }
}
