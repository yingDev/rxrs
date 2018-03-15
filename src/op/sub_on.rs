use std::marker::PhantomData;
use std::any::{Any};
use std::rc::Rc;
use std::sync::atomic::AtomicIsize;

use observable::*;
use subscriber::*;
use subref::SubRef;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use scheduler::Scheduler;

pub struct SubOnOp<Src, V, Sch> //where Src : Observable<'a, V>+Send+Sync, Sch: Scheduler+Send+Sync
{
    source: Arc<Src>,
    scheduler: Arc<Sch>,
    PhantomData: PhantomData<V>
}

pub trait ObservableSubOn<Src, V, Sch> where Src : Observable<'static, V>+Send+Sync, Sch: Scheduler+Send+Sync
{
    fn sub_on(self, scheduler: Arc<Sch>) -> SubOnOp<Src, V, Sch> ;
}

impl<Src, V, Sch> ObservableSubOn<Src, V, Sch> for Src where Src : 'static+Observable<'static, V>+Send+Sync, Sch: Scheduler+Send+Sync
{
    fn sub_on(self, scheduler: Arc<Sch>) -> SubOnOp<Src, V, Sch>
    {
        SubOnOp{ scheduler, PhantomData, source: Arc::new(self)  }
    }
}

impl<Src, V:'static+Send+Sync, Sch> Observable<'static, V> for SubOnOp<Src, V, Sch> where Src : 'static+Observable<'static, V>+Send+Sync, Sch: Scheduler+Send+Sync
{
    fn sub(&self, dest: impl Observer<V> + Send + Sync+'static) -> SubRef
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
        src.take(30).sub_on(NewThreadScheduler::get()).subf(|v| println!("next {} thread: {:?}", v, thread::current().id() ));

        thread::sleep(Duration::from_secs(10));
    }
}