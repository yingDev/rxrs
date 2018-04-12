use std::sync::Arc;
use scheduler::Scheduler;
use observable::*;
use scheduler::SchedulerObserveOn;
use util::mss::*;


pub trait ObservableObserveOn<'sa, Src, V:'static+Send+Sync, Sch, SrcSSO:?Sized, SrcSSS:?Sized, SSA:?Sized, SSS:?Sized>
{
    type ObserveOn;
    fn observe_on(self, scheduler: Arc<Sch>) -> Self::ObserveOn;
}

impl<'sa, Src, V:'static+Send+Sync, Sch, SrcSSO:?Sized, SrcSSS:?Sized, SSA:?Sized, SSS:?Sized> ObservableObserveOn<'sa, Src, V, Sch, SrcSSO, SrcSSS, SSA, SSS> for Src where
    Src: Observable<'sa, V, SrcSSO, SrcSSS>,
    Sch: SchedulerObserveOn<'sa, V, Src, SrcSSO, SrcSSS, SSA, SSS>
{
    type ObserveOn = Sch::ObserveOn;
    fn observe_on(self, scheduler: Arc<Sch>) -> Self::ObserveOn
    {
        scheduler.observe_on(self)
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    use op::*;
    use scheduler::*;
    use std::thread;
    use std::time::Duration;
    use std::sync::atomic::AtomicUsize;
    use test_fixture::SimpleObservable;

    #[test]
    fn basic()
    {
        let src = SimpleObservable;

        src.observe_on(NewThreadScheduler::get()).subf( |v| println!("{} on {:?}", v, ::std::thread::current().id() ) );

        thread::sleep(::std::time::Duration::from_millis(1000));
    }

}