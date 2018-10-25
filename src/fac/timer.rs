use std::time::Duration;
use crate::*;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::*;

//todo:

pub struct Timer<SS: YesNo, Sch: SchedulerPeriodic<SS>>
{
    period: Duration,
    scheduler: Arc<Sch>,
    PhantomData: PhantomData<SS>
}

impl<Sch: SchedulerPeriodic<YES>> Timer<YES, Sch>
{
    //todo: default should be a DefaultScheduler ...
    pub fn new(period: Duration, scheduler: Sch) -> Self
    {
        Timer{ period, scheduler: Arc::new(scheduler), PhantomData }
    }
}


impl<Sch: SchedulerPeriodic<YES>+Send+Sync+'static>
Observable<'static, YES, Val<usize>>
for Timer<YES, Sch>
{
    fn sub(&self, next: impl ActNext<'static, YES, Val<usize>>, ec: impl ActEc<'static, YES>) -> Unsub<'static, YES>
    {
        let next = sendsync_next(next);
        let count = AtomicUsize::new(0);
        //hack: avoid sch being dropped when Timer is dropped
        //todo: find a better way ?
        let sch = self.scheduler.clone();
        self.scheduler.schedule_periodic(self.period, move |()|{
            sch.as_ref();
            next.call(By::v(count.fetch_add(1, Ordering::Relaxed)));
        })
    }

    fn sub_dyn(&self, next: Box<ActNext<'static, YES, Val<usize>>>, ec: Box<ActEcBox<'static, YES>>) -> Unsub<'static, YES>
    { self.sub(dyn_to_impl_next_ss(next), dyn_to_impl_ec_ss(ec)) }
}

#[cfg(test)]
mod test
{
    use crate::*;
    use std::time::Duration;
    use std::sync::Arc;
    use std::rc::Rc;
    use std::cell::Cell;
    use std::sync::atomic::*;

    #[test]
    fn smoke()
    {
        let (n, n1) = Arc::new(AtomicUsize::new(0)).clones();
        let t = Timer::new(Duration::from_millis(33), NewThreadScheduler::new(Arc::new(DefaultThreadFac)));

        t.take(10).sub(move |v: By<_>| { n.store(*v, Ordering::SeqCst); }, ());
        assert_ne!(n1.load(Ordering::SeqCst), 9);


        ::std::thread::sleep_ms(1000);
        assert_eq!(n1.load(Ordering::SeqCst), 9);
    }
}