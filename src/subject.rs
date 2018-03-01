use std::any::Any;
use std::borrow::Cow;
use std::cell::RefCell;
use std::cell::Ref;
use std::marker::PhantomData;
use std::rc::Rc;

use util::*;

use observable::*;
use unsub_ref::*;
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

#[derive(Clone)]
struct Record<V>
{
    disposed: Arc<AtomicBool>,
    ob: Arc<Observer<V>+Send+Sync>,
    sub: UnsubRef<'static>
}

impl<V> Record<V>
{
    fn new(ob: Arc<Observer<V>+Send+Sync>, sub: UnsubRef<'static>, arc_dispose: Arc<AtomicBool>) -> Record<V>
    {
        Record{ disposed: arc_dispose, ob, sub  }
    }

    #[inline]
    fn next(&self, v:V)
    {
        if self.disposed.load(Ordering::Acquire) { return; }
        self.ob.next(v);
    }

    #[inline]
    fn complete(&self)
    {
        if self.disposed.compare_and_swap(false, true, Ordering::SeqCst) { return; }
        self.ob.complete();
        self.sub.unsub();
    }

    #[inline]
    fn err(&self, e:Arc<Any+Send+Sync>)
    {
        if self.disposed.compare_and_swap(false, true, Ordering::SeqCst) { return; }
        self.ob.err(e);
        self.sub.unsub();
    }
}

enum State<V:Clone>{
    Normal(Arc<NormalState<V>>), Faulted(Arc<Any+Send+Sync>), Completed
}

struct NormalState<V: Clone>
{
    obs: ArcCell<Vec< Record<V> >>,
    toAdd: Mutex<Vec<Record<V>>>,
    needs_seep: AtomicBool,
}

impl<V:Clone> NormalState<V>
{
    fn sweep(&self)
    {
        self.needs_seep.store(true, Ordering::SeqCst);

        loop{
            let mut guard = self.toAdd.lock().unwrap();
            let toAdd = guard.clone();
            guard.clear();
            ::std::mem::drop(guard);

            let (mut newObs, toRemove) :(Vec<Record<V>>, Vec<Record<V>>) = Arc::as_ref(&self.obs.get()).clone().into_iter().partition(|r| !r.disposed.load(Ordering::Acquire));
            for r in toRemove {
                println!("removing observer...");
                r.sub.unsub();
            }

            for r in toAdd{
                println!("add observer...");

                newObs.push(r);
            }

            self.obs.set(Arc::new(newObs));

            if self.needs_seep.compare_and_swap(true, false, Ordering::SeqCst) {
                break;
            }
        }
    }
}

pub struct Subject<V:Clone>
{
    _state: ArcCell<State<V>>,
}

impl<V:Clone> Drop for Subject<V>
{
    fn drop(&mut self)
    {
        match *self._state.get() {
            State::Normal(ref normal) => {
                normal.obs.get().iter().for_each(|r| r.sub.unsub() );
            },
            _ => {}
        }
    }
}

impl<'a,  V:'static+Clone> Subject<V>
{
    pub fn new() -> Subject<V>
    {
        Subject{ _state: ArcCell::new(Arc::new(
            State::Normal(
                Arc::new(NormalState{
                    obs: ArcCell::new(Arc::new(Vec::new())),
                    needs_seep: AtomicBool::new(false),
                    toAdd: Mutex::new(Vec::new())
                })
            ))
        ) }
    }

}

impl<V:Clone+'static> Observer<V> for Subject<V>
{
    fn next(&self, v:V)
    {
        match *self._state.get(){
            State::Normal(ref normal) => {
                normal.obs.get().iter().for_each(|o| { o.next(v.clone()); } );
                if normal.needs_seep.load(Ordering::SeqCst) {
                    normal.sweep();
                }
            },
            _ => {}
        }
    }

    fn err(&self, e:Arc<Any+Send+Sync>)
    {
        let oldState = self._state.set(Arc::new(State::Faulted(e.clone())));
        match *oldState {
            State::Normal(ref normal) => {
                normal.obs.get().iter().for_each(|o| o.err(e.clone() ));
            },
            _=> {}
        }
    }

    fn complete(&self)
    {
        let oldState = self._state.set(Arc::new(State::Completed));
        match *oldState {
            State::Normal(ref normal) => {
                println!("subject: complete");
                normal.obs.get().iter().for_each(|o| o.complete());
            },
            _=> {}
        }
    }

    fn _is_closed(&self) -> bool
    {
        match *self._state.get() {
            State::Normal(ref normal) => false,
            _ => true
        }
    }
}

impl<V:'static+Clone> Observable< V> for Subject<V>
{
    fn sub(&self, dest: Arc<Observer<V>+Send+Sync>) -> UnsubRef<'static>
    {
        if dest._is_closed() {
            return UnsubRef::empty();
        }

        match *self._state.get()
        {
            State::Normal(ref normal) => {
                let arc_unsub = Arc::new(AtomicBool::new(false));
                let arc_unsub2 = arc_unsub.clone();

                let stateClone = Arc::downgrade(&normal.clone());
                let sub = UnsubRef::fromFn(move ||{
                    let old = arc_unsub2.compare_and_swap(false, true, Ordering::SeqCst);
                    if !old {
                        if let Some(arc) = stateClone.upgrade() {
                            arc.sweep();
                        }
                    }
                });

                let rec = Record::new(dest, sub.clone(), arc_unsub);
                normal.toAdd.lock().unwrap().push(rec);
                normal.sweep();

                sub
            },
            State::Completed => {
                dest.complete();
                UnsubRef::empty()
            },
            State::Faulted(ref e) => {
                dest.err(e.clone());
                UnsubRef::empty()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::atomic::AtomicIsize;
    use super::*;

    #[test]
    fn basic()
    {
        let out = Arc::new(AtomicIsize::new(0));
        let out1 = out.clone();

        let subj = Subject::new();

        subj.subf(move |v| { out1.fetch_add(v, Ordering::SeqCst); }, |e| print!(" error "), || print!(" comp "));

        subj.next(1);
        subj.next(2);
        subj.complete();

        let result = out.load(Ordering::SeqCst);
        assert!(result == 3, format!("actual: {}", result));
    }

    #[test]
    fn threads()
    {
        use ::std::thread;

        let out = Arc::new(AtomicIsize::new(0));
        let out1 = out.clone();

        let subj = Arc::new(Subject::new());
        let (a,b,c) = (subj.clone(), subj.clone(), subj.clone());

        subj.subf(move |v| { out1.fetch_add(v, Ordering::SeqCst); }, |e| print!(" error "), || print!(" comp "));

        let mut handles = Vec::new();

        handles.push(thread::spawn(move || { print!("a"); a.next(1); }));
        handles.push(thread::spawn(move || { print!("b"); b.next(2); }));
        handles.push(thread::spawn(move || { print!("c"); c.next(3); }));

        for j in handles{
            j.join();
        }
        print!("d");
        subj.complete();

        subj.next(999);

        let result = out.load(Ordering::SeqCst);
        assert_eq!(result,6);
    }

    #[test]
    fn unsub()
    {
        use ::std::thread;

        let out = Arc::new(AtomicIsize::new(0));
        let out1 = out.clone();

        let subj = Arc::new(Subject::new());
        let (a,b,c) = (subj.clone(), subj.clone(), subj.clone());

        let un = subj.subf(move |v| { out1.fetch_add(v, Ordering::SeqCst); }, |e| println!(" error "), || println!(" comp "));
        subj.next(1);


        thread::spawn(move || un.unsub()).join();


        subj.next(2);
        subj.complete();

        subj.next(999);

        let result = out.load(Ordering::SeqCst);
        assert_eq!(result, 1);
    }

    #[test]
    fn unsub_on_drop()
    {
        let b = Arc::new(AtomicBool::new(false));
        let b1 = b.clone();

        {
            let s = Subject::<i32>::new();
            s.subn(|v|{}).add(move || b.store(true, Ordering::SeqCst));
        }

        assert_eq!(b1.load(Ordering::SeqCst), true);
    }

    #[test]
    fn unsub_on_comp()
    {
        let b = Arc::new(AtomicBool::new(false));
        let b1 = b.clone();

        let s = Subject::<i32>::new();
        s.subn(|v|{}).add(move || b.store(true, Ordering::SeqCst));
        s.next(123);
        s.complete();

        assert!(b1.load(Ordering::SeqCst) == true);
    }

    #[test]
    fn unsub_on_err()
    {
        let b = Arc::new(AtomicBool::new(false));
        let b1 = b.clone();

        let s = Subject::<i32>::new();
        s.subn(|v|{}).add(move || b.store(true, Ordering::SeqCst));
        s.next(123);
        s.err(Arc::new("this is error"));

        assert_eq!(b1.load(Ordering::SeqCst) , true);
    }
}