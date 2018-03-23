use std::marker::PhantomData;
use std::any::{Any};
use std::rc::Rc;
use std::sync::atomic::AtomicIsize;

use observable::*;
use subref::SubRef;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use scheduler::Scheduler;
use util::ArcCell;
use util::AtomicOption;
use std::sync::Weak;
use std::collections::VecDeque;
use std::sync::Condvar;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use observable::*;
use observable::RxNoti::*;
use util::mss::*;
use scheduler::SchedulerObserveOn;


pub trait ObservableObserveOn<'sa, Src, V:'static+Send+Sync, SrcSSO:?Sized, Sch, ObserveOn: Observable<'static, V, Sch::SSA>>
    where Src: Observable<'sa, V, SrcSSO>,
          Sch: Scheduler+SchedulerObserveOn<'sa, V, Src, SrcSSO, ObserveOn>
{
    fn observe_on(self, scheduler: Arc<Sch>) -> ObserveOn;
}

impl<'sa, Src, V:'static+Send+Sync, SrcSSO:?Sized, Sch, ObserveOn: Observable<'static, V, Sch::SSA>> ObservableObserveOn<'sa, Src, V, SrcSSO, Sch, ObserveOn> for Src
    where Src: Observable<'sa, V, SrcSSO>,
        Sch: Scheduler+SchedulerObserveOn<'sa, V, Src, SrcSSO, ObserveOn>
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