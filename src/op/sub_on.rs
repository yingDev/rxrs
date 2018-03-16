use std::marker::PhantomData;
use std::any::{Any};
use std::rc::Rc;
use std::sync::atomic::AtomicIsize;

use observable::*;
use subref::SubRef;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use scheduler::Scheduler;

pub struct SubOnOp<V, Sch>
{
    source: Arc<Observable<'static, V> + 'static + Send+ Sync>,
    scheduler: Arc<Sch>,
}

pub trait ObservableSubOn<V, Sch>
{
    fn sub_on(self, scheduler: Arc<Sch>) -> Arc<Observable<'static, V>+'static+Send+Sync>;
}

impl<V:'static+Send+Sync, Sch> ObservableSubOn<V, Sch> for Arc<Observable<'static, V>+'static+ Send+ Sync> where Sch: 'static + Scheduler+Send+Sync
{
    fn sub_on(self, scheduler: Arc<Sch>) -> Arc<Observable<'static, V>+'static+Send+Sync>
    {
        Arc::new(SubOnOp{ scheduler, source: self  })
    }
}

impl<V:'static+Send+Sync, Sch> Observable<'static, V> for SubOnOp<V, Sch> where Sch: 'static + Scheduler+Send+Sync
{
    fn sub(&self, dest: Arc<Observer<V> + Send + Sync+'static>) -> SubRef
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
        let src = rxfac::timer(100, Some(100), NewThreadScheduler::get());
        src.rx().take(30).sub_on(NewThreadScheduler::get()).subf(|v| println!("next {} thread: {:?}", v, thread::current().id() ));

        thread::sleep(Duration::from_secs(10));
    }
}