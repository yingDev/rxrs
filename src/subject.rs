use std::any::Any;
use std::borrow::Cow;
use std::cell::RefCell;
use std::cell::Ref;
use std::marker::PhantomData;
use std::rc::Rc;

use util::*;

use observable::*;
use subref::*;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::atomic::AtomicPtr;
use std::sync::atomic::AtomicUsize;
use std::slice::Iter;
use std::sync::ONCE_INIT;
use std::sync::Once;
use std::sync::Weak;
use std::cell::UnsafeCell;
use std::mem;

struct SubRecord<'a,V>
{
    o: Arc<Observer<V>+Send+Sync+'a>,
    sub: SubRef
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
    state: Arc<State<'a,V>>,
    x: UnsafeCell<Option<Arc<Observer<V>+Send+Sync+'a>>>
}

unsafe impl<'a,V> Send for Subject<'a, V>{}
unsafe impl<'a,V> Sync for Subject<'a, V>{}

struct State<'a, V>
{
    completed: AtomicBool,
    err: ArcCell<Option<Arc<Any+Send+Sync>>>,
    obs: ArcCell<Option<Vec<SubRecord<'a, V>>>>,
}

impl<'a, V> Subject<'a,V>
{
    #[inline(always)]
    pub fn new()-> Subject<'a,V>
    {
        Subject { x: UnsafeCell::new(None),  state: Arc::new(State{completed: AtomicBool::new(false), err: ArcCell::new(Arc::new(None)), obs: ArcCell::new(Arc::new(Some(Vec::new()))) }) }
    }
}

impl<'a, V> Drop for Subject<'a, V>
{
    #[inline(always)]
    fn drop(&mut self)
    {
        self.state.unsub(None);
    }
}

impl<'a,V> State<'a,V>
{
    #[inline(always)]
    fn unsub(&self, subref: Option<&SubRef>)
    {
        let mut new_obs = None;
        let mut vec = None;
        let mut old_obs;

        loop {
            old_obs = self.obs.get();

            if let &Some(ref obs) = &*old_obs {
                if subref.is_some() && vec.is_none(){
                    vec = Some(Vec::with_capacity(obs.len()));
                }

                if new_obs.is_none() {
                    new_obs = Some(Arc::new(None));
                }

                if subref.is_some() {
                    let rvec = vec.as_mut().unwrap();
                    rvec.clear();
                    for rec in obs.iter() {
                        if SubRef::ptr_eq(&rec.sub, subref.as_ref().unwrap()) {
                            rvec.push(rec.clone());
                        }
                    }
                }

                mem::swap(Arc::get_mut(new_obs.as_mut().unwrap()).unwrap(), &mut vec);

                if Arc::ptr_eq(&self.obs.compare_swap(old_obs.clone(), new_obs.clone().unwrap()), &old_obs) {
                    break;
                }

                mem::swap(Arc::get_mut(new_obs.as_mut().unwrap()).unwrap(), &mut vec);
            } else { return; }

        }

        if let &Some(ref obs) = &*old_obs  {
            for o in obs {
                o.sub.unsub();
            }
        }
    }
}

impl<'a, V> Observable<'a,V, Yes> for Subject<'a,V>
{
    fn sub(&self, o: impl Observer<V>+'a+Send+Sync) -> SubRef
    {
        let state = &self.state;

        let mut some_o = Some(o);
        let mut obs_vec= None;
        let mut observer: Option<Arc<Observer<V>+Send+Sync>> = None;
        let mut sub = None;
        let mut new_obs= None;

        loop {
            let old_obs = state.obs.get();

            if old_obs.is_none() {
                return SubRef::empty();
            }

            if let &Some(ref e) = &*state.err.get() {
                if let Some(o) = some_o { o.err(e.clone()); } else { observer.unwrap().err(e.clone()); }
                return SubRef::empty();
            }

            if state.completed.load(Ordering::SeqCst) {
                if let Some(o) = some_o { o.complete(); } else { observer.unwrap().complete(); }
                return SubRef::empty();
            }

            if obs_vec.is_none() {
                let weak_state: Weak<State<PhantomData<()>>> = unsafe { mem::transmute(Arc::downgrade(state)) };
                let _sub = SubRef::signal();
                let _sub2 = _sub.clone();
                _sub.add(SubRef::from_fn(box move || {
                    if let Some(s) = weak_state.upgrade() {
                        s.unsub(Some(&_sub2));
                    }
                }));
                sub = Some(_sub);
                observer = Some(Arc::new( some_o.take().unwrap()) );
                new_obs = Some(Arc::new(None));
                obs_vec = Some(Vec::with_capacity(Arc::as_ref(&old_obs).as_ref().unwrap().len()+1));
            }

            {
                let obs = obs_vec.as_mut().unwrap();
                obs.clear();
                obs.extend(Option::as_ref(&old_obs).unwrap().iter().map(|r| r.clone()));
                obs.push(SubRecord{ o: observer.clone().unwrap(), sub: sub.clone().unwrap() });
            }

            mem::swap(Arc::get_mut(new_obs.as_mut().unwrap()).unwrap(), &mut obs_vec);

            if Arc::ptr_eq(&state.obs.compare_swap(old_obs.clone(), new_obs.clone().unwrap()), &old_obs) {
                return sub.unwrap();
            }

            mem::swap(Arc::get_mut(new_obs.as_mut().unwrap()).unwrap(), &mut obs_vec);
        }
    }
}

impl<'a, V:Clone> Observer<V> for Subject<'a, V>
{
    #[inline(always)]
    fn next(&self, v: V)
    {
        if let &Some(ref obs) = &*self.state.obs.get() {
            for rec in obs.iter() {
                if ! rec.sub.disposed() {
                    rec.o.next(v.clone());
                }
            }
        }
    }

    #[inline(always)]
    fn err(&self, e: Arc<Any+Send+Sync>)
    {
        let state = &self.state;
        if state.completed.load(Ordering::SeqCst) { return; }
        if state.err.get().is_some() { return; }

        state.err.set(Arc::new(Some(e.clone())));

        let new_obs = Arc::new(None);
        let mut old_obs;
        loop {
            old_obs = state.obs.get();
            if Arc::ptr_eq(&state.obs.compare_swap(old_obs.clone(), new_obs.clone()), &old_obs) {
                break;
            }
        }

        if let &Some(ref obs) = &*old_obs {
            for rec in obs.iter() {
                rec.sub.unsub();
                rec.o.err(e.clone());
            }
        }
    }

    #[inline(always)]
    fn complete(&self)
    {
        let state = &self.state;
        if state.completed.compare_and_swap(true, false, Ordering::SeqCst) { return; }

        let mut old_obs;
        let new_obs = Arc::new(None);

        loop {
            old_obs = state.obs.get();
            if Arc::ptr_eq(&state.obs.compare_swap(old_obs.clone(), new_obs.clone()), &old_obs) {
                break;
            }
        }

        if let &Some(ref obs) = &*old_obs {
            for rec in obs.iter() {
                rec.sub.unsub();
                rec.o.complete();
            }
        }
    }

    #[inline(always)]
    fn _is_closed(&self) -> bool
    {
        self.state.completed.load(Ordering::Acquire) || self.state.err.get().is_some()
    }
}

#[cfg(test)]
mod test {

}