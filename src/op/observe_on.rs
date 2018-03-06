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
use util::ArcCell;
use util::AtomicOption;
use std::sync::Weak;
use std::collections::VecDeque;
use std::sync::Condvar;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

pub struct ObserveOn<Src, V, Sch> where Src : Observable<V>+Send+Sync, Sch: Scheduler+Send+Sync
{
    source: Arc<Src>,
    scheduler: Arc<Sch>,
    PhantomData: PhantomData<V>
}

struct ObserveOnState<V, Sch> where Sch: Scheduler+Send+Sync
{
    scheduler: Arc<Sch>,
    queue: Arc<(Condvar, Mutex<VecDeque<V>>, AtomicOption<Arc<Any+Send+Sync>>)>
}

pub trait ObservableObserveOn<Src, V, Sch> where Src : Observable<V>+Send+Sync, Sch: Scheduler+Send+Sync
{
    fn observe_on(self, scheduler: Arc<Sch>) -> ObserveOn<Src, V, Sch> ;
}

impl<Src, V, Sch> ObservableObserveOn<Src, V, Sch> for Src where Src : Observable<V>+Send+Sync, Sch: Scheduler+Send+Sync
{
    fn observe_on(self, scheduler: Arc<Sch>) -> ObserveOn<Src, V, Sch>
    {
        ObserveOn{ scheduler, PhantomData, source: Arc::new(self)  }
    }
}

impl<V: Send+Sync+'static, Sch> SubscriberImpl<V,ObserveOnState<V, Sch>> for Subscriber<V,ObserveOnState<V, Sch>> where Sch: Scheduler+Send+Sync+'static
{
    fn on_next(&self, v:V)
    {
        let &(ref cond, ref lock, ref err) = &*self._state.queue;
        lock.lock().unwrap().push_back(v);
        cond.notify_one();
    }

    fn on_err(&self, e:Arc<Any+Send+Sync>)
    {
        let &(ref cond, ref lock, ref err) = &*self._state.queue;
        {
            lock.lock().unwrap();
            err.swap(e, Ordering::Release);
        }
        cond.notify_one();
    }

    fn on_comp(&self)
    {
        let &(ref cond, ref lock, ref err) = &*self._state.queue;
        cond.notify_one();

    }
}

impl<Src, V:'static+Send+Sync, Sch> Observable< V> for ObserveOn<Src, V, Sch> where Src : 'static + Observable<V>+Send+Sync, Sch: Scheduler+Send+Sync+'static
{
    fn sub(&self, dest: Arc<Observer<V>+Send+Sync>) -> UnsubRef<'static>
    {
        let s = Arc::new(Subscriber::new(ObserveOnState {
            scheduler: self.scheduler.clone(),
            queue: Arc::new(( Condvar::new(), Mutex::new(VecDeque::new()), AtomicOption::new() ))
        }, dest, false)
        );

        let sig = UnsubRef::signal();
        let s2 = s.clone();

        sig.add(self.scheduler.schedule_long_running(sig.clone(), move ||{
            dispatch(s2);
        }));

        sig.add(self.source.sub(s));
        sig
    }
}

fn dispatch<V, Sch>(subscriber: Arc<Subscriber<V, ObserveOnState<V,Sch>>>) where Sch: Scheduler+Send+Sync
{
    let queue = subscriber._state.queue.clone();
    let dest = subscriber._dest.clone();

    loop {
        let &(ref cond, ref lock, ref err) = &*queue;

        while let Some(v) = lock.lock().unwrap().pop_front() {
            dest.next(v);
        }

        if subscriber.stopped() {
            if let Some(e) = err.take(Ordering::Acquire) {
                dest.err(e);
            }else {
                dest.complete();
            }
            return;
        }

        cond.wait(lock.lock().unwrap());
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    use subject::*;
    use fac::*;
    use op::*;
    use scheduler::*;
    use std::thread;

    #[test]
    fn basic()
    {
        rxfac::range(0..10).observe_on(Arc::new(NewThreadScheduler::new())).take(3).map(|v| format!("*{}*", v))
            .subf(|v| println!("{} on thread {:?}", v, thread::current().id()), (),
                  | | println!("complete on thread {:?}", thread::current().id()));

        thread::sleep(::std::time::Duration::from_millis(1000));
    }
}