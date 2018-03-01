use observable::*;
use std::cell::RefCell;
use std::cell::Ref;
use std::cell::RefMut;
use std::any::{Any};
use std::rc::Rc;
use std::marker::PhantomData;
use unsub_ref::*;
use std::sync::Arc;
use util::*;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

pub struct Subscriber<V, S, VOut=V>
{
    pub _state: S,
    pub _stopped: AtomicBool,
    pub _dest: Arc<Observer<VOut>+Send+Sync>,
    pub _sub: AtomicOption<UnsubRef<'static>>,
    PhantomData: PhantomData<V>
}

impl<V,S,VOut> Subscriber<V,S,VOut>
{
    pub fn new(state: S, dest: Arc<Observer<VOut>+Send+Sync>, stopped: bool) -> Subscriber<V, S,VOut>
    {
        Subscriber{ _state: state, _stopped: AtomicBool::new(stopped), _dest: dest, _sub: AtomicOption::new(), PhantomData }
    }
    #[inline] pub fn stopped(&self) -> bool { self._stopped.load(Ordering::SeqCst) }
    //#[inline] pub fn set_stopped(&self, stopped: bool) { self._stopped.compare_and_swap(false, true, Ordering::SeqCst) }
    #[inline]
    pub fn do_unsub(&self)
    {
        self._sub.take(Ordering::SeqCst).map(|s|s.unsub());
    }

    pub fn set_unsub(&self, s: &UnsubRef<'static>)
    {
        if ! s.disposed()
        {
            self._sub.swap(s.clone(), Ordering::SeqCst);
            if s.disposed() {
                self._sub.take(Ordering::Relaxed);
            }
        }
    }

}
pub trait SubscriberImpl<V, S> : Observer<V>
{
    fn on_next(&self,  v:V);
    fn on_err(&self, e:Arc<Any+Send+Sync>);
    fn on_comp(&self);
}

impl<V, S,VOut> Observer<V> for Subscriber<V, S,VOut> where Subscriber<V, S,VOut>: SubscriberImpl<V, S>
{
    fn next(&self, v: V)
    {
        if self._stopped.load(Ordering::SeqCst) { return; }
        self.on_next(v);
    }

    fn err(&self, e: Arc<Any+Send+Sync>)
    {
        if self._stopped.compare_and_swap(false, true, Ordering::SeqCst) { return; }
        self.on_err(e);
    }

    fn complete(&self)
    {
        if self._stopped.compare_and_swap(false, true, Ordering::SeqCst) { return; }
        self.on_comp();
    }

    fn _is_closed(&self) -> bool { return self._stopped.load(Ordering::SeqCst) }
}
