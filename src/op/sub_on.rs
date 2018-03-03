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

struct SubOnState
{
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
            println!("sub_on: thread {:?}", ::std::thread::current().id());
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

    #[test]
    fn basic()
    {
        println!("src thread: {:?}", ::std::thread::current().id());
        let src = Arc::new(rxfac::range(0..10));
        src.clone().take(3).sub_on(Arc::new(NewThreadScheduler::new())).subn(|v| println!("next {} thread: {:?}", v, ::std::thread::current().id() ));

        ::std::thread::sleep(::std::time::Duration::from_millis(1000));
    }
}