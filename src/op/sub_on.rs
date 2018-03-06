use std::marker::PhantomData;
use std::any::{Any};
use std::rc::Rc;
use std::sync::atomic::AtomicIsize;

use observable::*;
use subscriber::*;
use unsub_ref::UnsubRef;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use scheduler::Scheduler;

pub struct SubOnOp<Src, V, Sch> where Src : Observable<V>+Send+Sync, Sch: Scheduler+Send+Sync
{
    source: Arc<Src>,
    scheduler: Arc<Sch>,
    PhantomData: PhantomData<V>
}

pub trait ObservableSubOn<Src, V, Sch> where Src : Observable<V>+Send+Sync, Sch: Scheduler+Send+Sync
{
    fn sub_on(self, scheduler: Arc<Sch>) -> SubOnOp<Src, V, Sch> ;
}

impl<Src, V, Sch> ObservableSubOn<Src, V, Sch> for Src where Src : Observable<V>+Send+Sync, Sch: Scheduler+Send+Sync
{
    fn sub_on(self, scheduler: Arc<Sch>) -> SubOnOp<Src, V, Sch>
    {
        SubOnOp{ scheduler, PhantomData, source: Arc::new(self)  }
    }
}

impl<Src, V:'static+Send+Sync, Sch> Observable< V> for SubOnOp<Src, V, Sch> where Src : 'static + Observable<V>+Send+Sync, Sch: Scheduler+Send+Sync
{
    fn sub(&self, dest: Arc<Observer<V>+Send+Sync>) -> UnsubRef<'static>
    {
        let src = self.source.clone();
        self.scheduler.schedule(move ||{
            src.sub(dest)
        })
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    use subject::*;
    use scheduler::*;
    use op::*;
    use fac::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn basic()
    {
        println!("src thread: {:?}", thread::current().id());
        let src = Arc::new(rxfac::timer(100, Some(100), NewThreadScheduler::get()));
        src.take(30).sub_on(NewThreadScheduler::get()).subn(|v| println!("next {} thread: {:?}", v, thread::current().id() ));

        thread::sleep(Duration::from_secs(10));
    }
}