use std::sync::Once;
use std::sync::ONCE_INIT;
use std::sync::Arc;
use std::sync::atomic::{ AtomicBool};
use std::sync::atomic::Ordering;

use util::ArcCell;
use std::boxed::FnBox;
use std::rc::Rc;
use std::marker::PhantomData;
use util::mss::*;

pub struct SubRef<SS:?Sized>
{
    state: Arc<State>,
    PhantomData: PhantomData<*const SS>
}

pub struct InnerSubRef<SS:?Sized>
{
    state: Arc<State>,
    PhantomData: PhantomData<*const SS>
}

impl<SS:?Sized> Into<SubRef<SS>> for InnerSubRef<SS>
{
    fn into(self) -> SubRef<SS>
    {
        SubRef{ state: self.state, PhantomData }
    }
}
impl Into<SubRef<No>> for InnerSubRef<Yes>
{
    fn into(self) -> SubRef<No>
    {
        SubRef{ state: self.state, PhantomData }
    }
}

impl<SS:?Sized> Clone for SubRef<SS>
{
    fn clone(&self) -> SubRef<SS>
    {
        SubRef{ state: self.state.clone(), PhantomData}
    }
}
impl<SS:?Sized> Clone for InnerSubRef<SS>
{
    fn clone(&self) -> InnerSubRef<SS>
    {
        InnerSubRef{ state: self.state.clone(), PhantomData}
    }
}
unsafe impl Send for InnerSubRef<Yes> {}
unsafe impl Sync for InnerSubRef<Yes> {}

unsafe impl Send for SubRef<Yes> {}
unsafe impl Sync for SubRef<Yes> {}

struct State
{
    disposed: AtomicBool,
    cb: ArcCell<Box<FnBox()>>,
    extra: ArcCell<Vec<Arc<State>>>,
}

impl State
{
    fn unsub(&self)
    {
        if self.disposed.compare_and_swap(false, true, Ordering::AcqRel) {
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
        loop{
            let old = self.extra.get();
            old.iter().for_each(|s| s.unsub());
            if Arc::ptr_eq(&self.extra.compare_swap(old.clone(), empty_extra()), &old) {
                break;
            }
        }
    }
}

impl<SS:?Sized> SubRef<SS>
{
    pub fn empty() -> SubRef<SS> { InnerSubRef::empty().into() }
    pub fn unsub(&self) { self.state.unsub() }
}

impl<SS:?Sized> InnerSubRef<SS>
{
    #[inline(always)]
    pub fn ptr_eq(&self, other: &InnerSubRef<SS>) -> bool
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

    pub fn signal() -> InnerSubRef<SS>
    {
        InnerSubRef { state: Arc::new(
            State{
                disposed: AtomicBool::new(false),
                cb: ArcCell::new(empty_cb()),
                extra: ArcCell::new(empty_extra())
            }), PhantomData
        }
    }

    #[inline(never)]
    pub fn empty() -> InnerSubRef<SS>
    {
        static mut EMPTY: Option<Arc<State>> = None;
        static EMPTY_INIT: Once = ONCE_INIT;

        unsafe {
            EMPTY_INIT.call_once(|| {
                EMPTY = Some( Arc::new(
                    State{cb: ArcCell::new(empty_cb()), extra: ArcCell::new(empty_extra()), disposed: AtomicBool::new(true) },
                ));
            });
            InnerSubRef{ state: EMPTY.as_ref().unwrap().clone(), PhantomData }
        }
    }

    pub fn add(&self, un: SubRef<SS>)
    {
        if Arc::ptr_eq(&un.state, &self.state) {return;}
        if un.state.disposed.load(Ordering::Acquire) { return; }

        let state = self.state.clone();
        let mut new = None;

        loop{
            let old = state.extra.get();
            let len = old.len() + 1;
            let new = new.get_or_insert_with(|| Arc::new(Vec::with_capacity(len)));

            {
                let new = Arc::get_mut(new).unwrap();
                new.clear();
                new.reserve(len);
                for s in old.iter() { new.push(s.clone()); }
                new.push(un.state.clone());
            }

            if un.state.disposed.load(Ordering::Acquire) { return; }
            if Arc::ptr_eq(&state.extra.compare_swap(old.clone(), new.clone()), &old) {
                break;
            }
        }
        if state.disposed.load(Ordering::Acquire) {
            state._unsub_extra();
        }
    }
}

impl InnerSubRef<Yes>
{
    pub fn new<F>(f: F) -> InnerSubRef<Yes> where F: 'static+FnBox()+Send+Sync
    {
        InnerSubRef { state: Arc::new(
            State{
                disposed: AtomicBool::new(false),
                cb: ArcCell::new(Arc::new(box f)),
                extra: ArcCell::new(empty_extra())
            }), PhantomData
        }
    }

    pub fn added(self, un: SubRef<Yes>) -> Self
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
        self.add(InnerSubRef::<Yes>::new(f).into());
    }
}

impl InnerSubRef<No>
{
    pub fn new<F>(f: F) -> InnerSubRef<No> where F: 'static+FnBox()
    {
        InnerSubRef { state: Arc::new(
            State{
                disposed: AtomicBool::new(false),
                cb: ArcCell::new(Arc::new(box f)),
                extra: ArcCell::new(empty_extra())
            }), PhantomData
        }
    }


    pub fn addss(&self, un: SubRef<Yes>)
    {
        if un.state.disposed.load(Ordering::Acquire) { return; }

        let mut old = self.state.extra.set(empty_extra());
        let mut new : Arc<Vec<Arc<State>>> = if Arc::ptr_eq(&old, &empty_extra()) {
            Arc::new(Vec::with_capacity(old.len() + 1))
        } else { old };

        if let Some(mut new) = Arc::get_mut(&mut new) {
            new.push(un.state);
        }

        self.state.extra.set(new);
    }

    pub fn added(self, un: SubRef<No>) -> Self
    {
        self.add(un);
        self
    }

    pub fn addedss(self, un: SubRef<Yes>) -> Self
    {
        self.addss(un);
        self
    }

    pub fn addedf<F>(self, f: F) -> Self where F: 'static+FnBox()
    {
        self.addf(f);
        self
    }

    pub fn addf<F>(&self, f: F) where F: 'static+FnBox()
    {
        self.add(InnerSubRef::<No>::new(f).into());
    }
}


pub trait IntoSubRef<SS:?Sized>
{
    fn into(self) -> SubRef<SS>;
}
impl IntoSubRef<Yes> for ()
{
    #[inline(always)]
    fn into(self) -> SubRef<Yes> { SubRef::empty() }
}

impl<SS:?Sized> IntoSubRef<SS> for SubRef<SS>
{
    #[inline(always)]
    fn into(self) -> SubRef<SS>{ self }
}



fn empty_extra() -> Arc<Vec<Arc<State>>>
{
    unsafe {
        static mut VALUE: Option<Arc<Vec<Arc<State>>>> = None;
        static INIT: Once = ONCE_INIT;

        INIT.call_once(||{
            VALUE = Some(Arc::new(Vec::new()))
        });
        VALUE.clone().unwrap()
    }
}

fn empty_cb() -> Arc<Box<FnBox()>>
{
    unsafe {
        static mut VALUE: Option<Arc<Box<FnBox()>>> = None;
        static INIT: Once = ONCE_INIT;

        INIT.call_once(||{
            VALUE = Some(Arc::new(box ||{}))
        });
        VALUE.clone().unwrap()
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::thread;
    use observable::*;
    use std::sync::Mutex;

    #[test]
    fn basic()
    {
        let a = InnerSubRef::<No>::new(||{});
        a.add(InnerSubRef::<No>::new(||{}).into());
    }

    #[test]
    fn disposed()
    {
        let u = InnerSubRef::<No>::new(||{});
        u.unsub();
        assert!(u.disposed());
    }

    #[test]
    fn add_after_unsub()
    {
        let u = InnerSubRef::<Yes>::new(||{});
        u.unsub();

        let v = Arc::new(AtomicBool::new(false));
        let v2 = v.clone();
        u.add(InnerSubRef::<Yes>::new(move || v2.store(true, Ordering::SeqCst)).into());

        assert!(v.load(Ordering::SeqCst));
    }

    #[test]
    fn threads()
    {
        let out = Arc::new(Mutex::new("".to_owned()));
        let (out2,out3) = (out.clone(), out.clone());

        let u = InnerSubRef::<Yes>::new(||{});
        let (u2,u3) = (u.clone(), u.clone());

        let j = thread::spawn(move || {
            u2.add(InnerSubRef::<Yes>::new(move || {  out.lock().unwrap().push_str("A");  }).into());
        });
        u3.add(InnerSubRef::<Yes>::new(move || { out2.lock().unwrap().push_str("A"); }).into());


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

        let s = InnerSubRef::<Yes>::new(move || o2.lock().unwrap().push('A'));
        let s2 = s.clone();

        s.addf(move || s2.addf(move || o3.lock().unwrap().push('C')));
        s.addf(move || o4.lock().unwrap().push('B'));

        s.unsub();

        assert_eq!(*out.lock().unwrap(), "ABC");
    }

}