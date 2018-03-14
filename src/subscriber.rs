use observable::*;
use std::cell::RefCell;
use std::cell::Ref;
use std::cell::RefMut;
use std::any::{Any};
use std::rc::Rc;
use std::marker::PhantomData;
use subref::*;
use std::sync::Arc;
use util::*;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

pub struct Subscriber<'a, V, S, Dest, VOut=V>
{
    pub _state: S,
    pub _stopped: AtomicBool,
    pub _dest: Dest,
    pub _sub: SubRef,
    PhantomData: PhantomData<(*const V,*const VOut, &'a())>
}
unsafe impl<'a, V, S, Dest, VOut> Send for Subscriber<'a, V, S, Dest, VOut> where Dest : Observer<VOut>+Send+Sync+'a, S:Send+Sync+'a{}
unsafe impl<'a, V, S, Dest, VOut> Sync for Subscriber<'a, V, S, Dest, VOut> where Dest : Observer<VOut>+Send+Sync+'a, S:Send+Sync+'a{}

impl<'a, 'b:'a, V,S,Dest,VOut> Subscriber<'a, V,S,Dest, VOut> where Dest : Observer<VOut>+Send+Sync+'a, S:Send+Sync+'b, Self: SubscriberImpl<V, S>+'b
{
    pub fn new(state: S, dest: Dest, stopped: bool) -> Subscriber<'a, V, S, Dest, VOut>
    {
        Subscriber{ _state: state, _stopped: AtomicBool::new(stopped), _dest: dest, _sub: SubRef::signal(), PhantomData }
    }
    #[inline] pub fn stopped(&self) -> bool { self._stopped.load(Ordering::SeqCst) }

    #[inline]
    pub fn do_unsub(&self)
    {
        self._sub.unsub();
    }

    pub fn do_sub(self, src: &impl Observable<'a,V>) -> SubRef
    {
        let _sub = self._sub.clone();
        let sub = src.sub(self);

        if sub.disposed() {
            _sub.unsub();
            return sub;
        }

        if _sub.disposed() {
            sub.unsub();
            return sub;
        }

        _sub.add(sub);
        _sub
    }

}
pub trait SubscriberImpl<V, S> : Observer<V>
{
    fn on_next(&self,  v:V);
    fn on_err(&self, e:Arc<Any+Send+Sync>);
    fn on_comp(&self);
}

impl<'a, V, S, Dest, VOut> Observer<V> for Subscriber<'a, V, S,Dest,VOut> where Subscriber<'a, V, S,Dest,VOut>: SubscriberImpl<V, S>, Dest : Observer<VOut>+Send+Sync+'a, S:Send+Sync+'a
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
