use std::sync::Arc;
use scheduler::Scheduler;
use observable::*;
use scheduler::SchedulerObserveOn;


pub trait ObservableObserveOn<'sa, Src, V:'static+Send+Sync, SSA:?Sized+'static, SrcSSO:?Sized, Sch, ObserveOn: Observable<'static, V, SSA>>
    where Src: Observable<'sa, V, SrcSSO>,
          Sch: SchedulerObserveOn<'sa, V, Src, SSA, SrcSSO, ObserveOn>
{
    fn observe_on(self, scheduler: Arc<Sch>) -> ObserveOn;
}

impl<'sa, Src, V:'static+Send+Sync, SSA:?Sized+'static, SrcSSO:?Sized, Sch, ObserveOn: Observable<'static, V, SSA>> ObservableObserveOn<'sa, Src, V, SSA, SrcSSO, Sch, ObserveOn> for Src
    where Src: Observable<'sa, V, SrcSSO>,
          Sch: SchedulerObserveOn<'sa, V, Src, SSA, SrcSSO, ObserveOn>
{
    fn observe_on(self, scheduler: Arc<Sch>) -> ObserveOn
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