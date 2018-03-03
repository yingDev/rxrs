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

pub struct ObserveOn<Src, V, Sch> where Src : Observable<V>+Send+Sync, Sch: Scheduler+Send+Sync
{
    source: Arc<Src>,
    scheduler: Arc<Sch>,
    PhantomData: PhantomData<V>
}

struct ObserveOnState<V, Sch> where Sch: Scheduler+Send+Sync
{
    scheduler: Arc<Sch>,
    subscriber: Weak<Subscriber<V, ObserveOnState<V, Sch>>>
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
        if let Some(me) = Weak::upgrade(&self._state.subscriber) {
            self._state.scheduler.schedule(move ||{
                me._dest.next(v);
                UnsubRef::empty()
            });
        }

    }

    fn on_err(&self, e:Arc<Any+Send+Sync>)
    {
        if let Some(me) = Weak::upgrade(&self._state.subscriber) {
            self._state.scheduler.schedule(move ||{
                me._dest.err(e);
                me.do_unsub();
                UnsubRef::empty()
            });
        }
    }

    fn on_comp(&self)
    {
        if let Some(me) = Weak::upgrade(&self._state.subscriber) {
            self._state.scheduler.schedule(move ||{
                me._dest.complete();
                me.do_unsub();
                UnsubRef::empty()
            });
        }

    }
}

impl<Src, V:'static+Send+Sync, Sch> Observable< V> for ObserveOn<Src, V, Sch> where Src : 'static + Observable<V>+Send+Sync, Sch: Scheduler+Send+Sync+'static
{
    fn sub(&self, dest: Arc<Observer<V>+Send+Sync>) -> UnsubRef<'static>
    {
        let mut s = Arc::new(Subscriber::new(ObserveOnState{ scheduler: self.scheduler.clone(), subscriber: Weak::new() }, dest, false));
        let weak = Arc::downgrade(&s);

        unsafe { *((&s._state.subscriber as *const _) as *mut _) = weak; }

        self.source.sub(s)
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