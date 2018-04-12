use std::any::Any;
use std::rc::Rc;
use observable::*;
use subref::*;
use std::cell::UnsafeCell;
use std::mem;
use util::mss::*;
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Weak;
use std::marker::PhantomData;
use std::sync::Once;
use std::sync::ONCE_INIT;

struct SubRecord<'a,V>
{
    o: Rc<Observer<V>+'a>,
    sub: InnerSubRef<No>
}

impl<'a,V> Clone for SubRecord<'a, V>
{
    #[inline(always)]
    fn clone(&self) -> SubRecord<'a,V>
    {
        SubRecord{ o: self.o.clone(), sub: self.sub.clone() }
    }
}

pub struct Subject<'a, V:'a>
{
    state: Rc<State<'a,V>>,
    stopped: Cell<bool>,
    x: UnsafeCell<Option<Rc<Observer<V>+'a>>>
}

struct State<'a, V>
{
    completed: Cell<bool>,
    err: RefCell<Option<ArcErr>>,
    obs: RefCell<Rc<Vec<SubRecord<'a, V>>>>,
}

fn none_obs<'a,V>() -> Rc<Vec<SubRecord<'a,V>>>
{
    static mut EMPTY: Option<Rc<Vec<SubRecord<'static,()>>>> = None;
    static EMPTY_INIT: Once = ONCE_INIT;

    unsafe {
        EMPTY_INIT.call_once(|| {
            EMPTY = Some( Rc::new(Vec::new()));
        });

        mem::transmute(EMPTY.clone().unwrap())
    }
}
fn default_obs<'a,V>() -> Rc<Vec<SubRecord<'a,V>>>
{
    static mut EMPTY: Option<Rc<Vec<SubRecord<'static,()>>>> = None;
    static EMPTY_INIT: Once = ONCE_INIT;

    unsafe {
        EMPTY_INIT.call_once(|| {
            EMPTY = Some( Rc::new(Vec::new()));
        });

        mem::transmute(EMPTY.clone().unwrap())
    }
}

impl<'a, V> Subject<'a,V>
{
    #[inline(always)]
    pub fn new()-> Subject<'a,V>
    {
        Subject { x: UnsafeCell::new(None),  state: Rc::new(State{completed: Cell::new(false), err: RefCell::new(None), obs: RefCell::new(default_obs()) }), stopped: Cell::new(false) }
    }

    #[inline(always)]
    fn is_stopped(&self) -> bool
    {
        if self.stopped.get(){ return true;}

        let state = &self.state;
        self.stopped.replace(
            if state.completed.get() { true }
            else if state.err.borrow().is_some(){ true}
            else if Rc::ptr_eq(&state.obs.borrow(), &none_obs()) { true }
            else { false }
        );

        return self.stopped.get();
    }
}

impl<'a, V> Drop for Subject<'a, V>
{
    #[inline(always)]
    fn drop(&mut self)
    {
        self.stopped.set(true);
        self.state.unsub(None);
    }
}

impl<'a,V> State<'a,V>
{
    #[inline(always)]
    fn unsub(&self, subref: Option<&InnerSubRef<No>>)
    {
        if let Some(sub) = subref {
            let obs = self.obs.borrow().iter().filter(|rec| !InnerSubRef::ptr_eq(&rec.sub, &sub)).map(|r| r.clone()).collect();
            self.obs.replace(Rc::new(obs));
            sub.unsub();
        } else {
            for rec in self.obs.replace(none_obs()).iter()  {
                rec.sub.unsub();
            }
        }
    }
}

impl<'a, V> Observable<'a,V, No, No> for Subject<'a,V>
{
    fn sub(&self, o: Mss<No, impl Observer<V>+'a>) -> SubRef<No>
    {
        if self.stopped.get() { return SubRef::empty(); }

        let state = &self.state;
        if state.completed.get() {
            o.complete();
            return SubRef::empty();
        }
        if let Some(e) = state.err.borrow().clone() {
            o.err(e);
            return SubRef::empty();
        }
        if self.is_stopped() {
            return SubRef::empty();
        }

        let mut obs = if Rc::ptr_eq(&state.obs.borrow(), &default_obs()) {
            Rc::new(Vec::with_capacity(state.obs.borrow().len()))
        } else { state.obs.replace(none_obs()) };

        let o = Rc::new(o.into_inner());
        let sub = InnerSubRef::<No>::signal();
        let rec = SubRecord{ o, sub: sub.clone()};
        Rc::get_mut(&mut obs).unwrap().push(rec);

        state.obs.replace(obs);

        let state : Weak<State<PhantomData<()>>> = unsafe { mem::transmute(Rc::downgrade(&state)) };
        sub.addf(byclone!( sub => move ||{
            if let Some(state) = state.upgrade() {
                state.unsub(Some(&sub));
            }
        }));

        sub.into_subref()
    }
}

impl<'a, V:Clone> Observer<V> for Subject<'a, V>
{
    #[inline(always)]
    fn next(&self, v: V)
    {
        if self.is_stopped() { return; }

        let obs = self.state.obs.borrow().clone();
        for rec in obs.iter() {
            if !rec.sub.disposed() {
                rec.o.next(v.clone());
            }
        }
    }

    #[inline(always)]
    fn err(&self, e: ArcErr)
    {
        if self.is_stopped() { return; }
        self.state.err.replace(Some(e.clone()));

        let obs = self.state.obs.borrow().clone();
        for rec in obs.iter() {
            rec.sub.unsub();
            rec.o.err(e.clone());
        }
    }

    #[inline(always)]
    fn complete(&self)
    {
        if self.is_stopped() { return; }
        self.state.completed.replace(true);

        let obs = self.state.obs.borrow().clone();
        for rec in obs.iter() {
            rec.sub.unsub();
            rec.o.complete();
        }
    }

    #[inline(always)]
    fn _is_closed(&self) -> bool
    {
        self.state.completed.get() || self.state.err.borrow().is_some()
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use op::*;

    #[test]
    fn basic()
    {
        let mut out = 0;

        {
            let n = 3;
            let s = Subject::new();

            s.rx().subf(
                |v| out += v
            );

            s.next(1);
            s.next(2);
            //s.complete();
        }

        assert_eq!(3, out);
    }
}