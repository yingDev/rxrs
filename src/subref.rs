use std::sync::Once;
use std::sync::ONCE_INIT;
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{ AtomicBool, AtomicUsize};
use std::sync::atomic::Ordering;
use std::collections::LinkedList;

use util::ArcCell;
use std::marker::PhantomData;
use util::mss::*;
use std::boxed::FnBox;
use observable::FnCell;
use std::mem;
use std::ptr;

pub struct SubRef
{
    state: Arc<State>
}

impl Clone for SubRef
{
    fn clone(&self) -> SubRef
    {
        SubRef{ state: self.state.clone()}
    }
}

unsafe impl Send for SubRef {}
unsafe impl Sync for SubRef {}

struct State
{
    disposed: AtomicBool,
    cb: ArcCell<Box<FnBox()+Send+Sync>>,
    extra: ArcCell<LinkedList<Arc<State>>>,
}

impl State
{
    fn unsub(&self)
    {
        if self.disposed.compare_and_swap(false, true, Ordering::Release) {
            return;
        }

        let cb = self.cb.set(empty_cb());
        if ! Arc::ptr_eq(&cb, &empty_cb()) {
            Arc::try_unwrap(cb).ok().unwrap().call_box(());
        }

        self._unsub_extra();
    }

    fn _unsub_extra(&self)
    {
        let mut old;
        loop{
            old = self.extra.get();
            old.iter().for_each(|s| s.unsub());
            if Arc::ptr_eq(&self.extra.compare_swap(old.clone(), empty_extra()), &old) {
                break;
            }
        }

    }
}

impl SubRef
{
    pub fn new<F>(f: F) -> SubRef where F: 'static+FnBox()+Send+Sync
    {
        SubRef { state: Arc::new(
            State{
                disposed: AtomicBool::new(false),
                cb: ArcCell::new(Arc::new(box f)),
                extra: ArcCell::new(empty_extra())
            }),
        }
    }

    pub fn signal() -> SubRef
    {
        SubRef { state: Arc::new(
            State{
                disposed: AtomicBool::new(false),
                cb: ArcCell::new(empty_cb()),
                extra: ArcCell::new(empty_extra())
            }),
        }
    }

    #[inline(never)]
    pub fn empty() -> SubRef
    {
        static mut EMPTY: Option<Arc<State>> = None;
        static EMPTY_INIT: Once = ONCE_INIT;

        unsafe {
            EMPTY_INIT.call_once(|| {
                EMPTY = Some( Arc::new(
                    State{cb: ArcCell::new(empty_cb()), extra: ArcCell::new(empty_extra()), disposed: AtomicBool::new(true) },
                ));
            });
            SubRef{ state: EMPTY.as_ref().unwrap().clone() }
        }
    }

    pub fn add(&self, un: SubRef)
    {
        if Arc::ptr_eq(&un.state, &self.state) {return;}
        if un.state.disposed.load(Ordering::Acquire) { return; }

        let state = self.state.clone();
        let mut new = Arc::new(LinkedList::new());

        loop{
            if un.state.disposed.load(Ordering::Acquire) { return; }

            let old = state.extra.get();
            {
                let mut newlst = Arc::get_mut(&mut new).unwrap();
                newlst.clear();
                newlst.extend(old.iter().map(|s| s.clone()));
                newlst.push_back(un.state.clone());
            }

            if Arc::ptr_eq(&state.extra.compare_swap(old.clone(), new.clone()), &old) {
                break;
            }
        }
        if un.state.disposed.load(Ordering::Acquire) { return; }
        if state.disposed.load(Ordering::SeqCst) {
            state._unsub_extra();
        }
    }

    pub fn added(self, un: SubRef) -> Self
    {
        self.add(un);
        self
    }

    pub fn addedf<F>(self, f: F) -> Self where F: 'static+FnBox()+Send+Sync
    {
        self.addf(f);
        self
    }

    pub fn addf<F>(&self, f: F) where F: 'static+FnBox()+Send+Sync
    {
        self.add(SubRef::new(f));
    }

    #[inline(always)]
    pub fn ptr_eq(&self, other: &SubRef) -> bool
    {
        Arc::ptr_eq(&self.state, &other.state)
    }

    #[inline(never)]
    pub fn unsub(&self)
    {
        self.state.unsub();
    }

    #[inline(always)]
    pub fn disposed(&self) -> bool { self.state.disposed.load(Ordering::SeqCst)}
}

pub trait IntoSubRef
{
    fn into(self) -> SubRef;
}
impl IntoSubRef for ()
{
    #[inline(always)]
    fn into(self) -> SubRef { SubRef::empty() }
}

impl IntoSubRef for SubRef
{
    #[inline(always)]
    fn into(self) -> SubRef{ self }
}

fn empty_extra() -> Arc<LinkedList<Arc<State>>>
{
    unsafe {
        static mut VALUE: Option<Arc<LinkedList<Arc<State>>>> = None;
        static INIT: Once = ONCE_INIT;

        INIT.call_once(||{
            VALUE = Some(Arc::new(LinkedList::new()))
        });
        unsafe{ VALUE.clone().unwrap() }
    }
}

fn empty_cb() -> Arc<Box<FnBox()+Send+Sync>>
{
    unsafe {
        static mut VALUE: Option<Arc<Box<FnBox()+Send+Sync>>> = None;
        static INIT: Once = ONCE_INIT;

        INIT.call_once(||{
            VALUE = Some(Arc::new(box ||{}))
        });
        unsafe{ VALUE.clone().unwrap() }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::thread;
    use observable::*;

    #[test]
    fn basic()
    {
        let a = SubRef::new(||{});
        a.add(SubRef::new(||{}));
    }

    #[test]
    fn disposed()
    {
        let u = SubRef::new(||{});
        u.unsub();
        assert!(u.disposed());
    }

    #[test]
    fn add_after_unsub()
    {
        let u = SubRef::new(||{});
        u.unsub();

        let v = Arc::new(AtomicBool::new(false));
        let v2 = v.clone();
        u.add(SubRef::new(move || v2.store(true, Ordering::SeqCst)));

        assert!(v.load(Ordering::SeqCst));
    }

    #[test]
    fn threads()
    {
        let out = Arc::new(Mutex::new("".to_owned()));
        let (out2,out3) = (out.clone(), out.clone());

        let u = SubRef::new(||{});
        let (u2,u3) = (u.clone(), u.clone());

        let j = thread::spawn(move || {
            u2.add(SubRef::new(move || {  out.lock().unwrap().push_str("A");  }));
        });
        u3.add(SubRef::new(move || { out2.lock().unwrap().push_str("A"); }));


        j.join();
        let j2  = thread::spawn(move || { u.unsub(); });
        j2.join();

        assert_eq!(*out3.lock().unwrap(), "AA");
    }

    #[test]
    fn add_during_unsub()
    {
        let out = Arc::new(Mutex::new(String::new()));
        let (o2,o3,o4) = (out.clone(), out.clone(), out.clone());

        let s = SubRef::new(move || o2.lock().unwrap().push('A'));
        let s2 = s.clone();

        s.addf(move || s2.addf(move || o3.lock().unwrap().push('C')));
        s.addf(move || o4.lock().unwrap().push('B'));

        s.unsub();

        assert_eq!(*out.lock().unwrap(), "ABC");
    }

}