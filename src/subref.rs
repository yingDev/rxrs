use std::sync::Once;
use std::sync::ONCE_INIT;
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::collections::LinkedList;
use util::AtomicOption;

use util::ArcCell;
use std::marker::PhantomData;
use util::mss::*;
use std::boxed::FnBox;
use observable::FnCell;
use std::mem;

static mut EMPTY_SUB_REF: Option<Arc<State>> = None;
static EMPTY_SUB_REF_INIT: Once = ONCE_INIT;

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
    cb: AtomicOption<Box<FnBox()+Send+Sync>>,
    extra: AtomicOption<LinkedList<Arc<State>>>,
}

impl State
{
    fn unsub(&self)
    {
        if self.disposed.compare_and_swap(false, true, Ordering::SeqCst){ return; }
        if let Some(mut cb) = self.cb.take(Ordering::Acquire) {
            cb.call_box(());
        }

        loop{
            if let Some(lst) = self.extra.take(Ordering::SeqCst){
                lst.iter().for_each(|f| f.unsub());
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
                cb: AtomicOption::with(box f),
                extra: AtomicOption::with(LinkedList::new()),
            }),
        }
    }

    pub fn add(&self, un: SubRef) -> &Self
    {
        if Arc::ptr_eq(&un.state, &self.state) {return self;}

        loop {
            if un.disposed() {
                break;
            }else if self.state.disposed.load(Ordering::SeqCst) {
                un.unsub();
                break;
            }else if let Some(mut lst) = self.state.extra.take(Ordering::SeqCst) {
                lst.push_back(un.state.clone());
                self.state.extra.swap(lst, Ordering::SeqCst);
                break;
            }
        }
        return self;
    }

    pub fn addf<F>(&self, f: F) where F: 'static+FnBox()+Send+Sync
    {
        self.add(SubRef::new(f));
    }

    pub fn signal() -> SubRef
    {
        SubRef { state: Arc::new(
            State{
                disposed: AtomicBool::new(false),
                cb: AtomicOption::new(),
                extra: AtomicOption::with(LinkedList::new())
            }),
        }
    }

    #[inline(never)]
    pub fn empty() -> SubRef
    {
        unsafe {
            EMPTY_SUB_REF_INIT.call_once(|| {
                //todo
                EMPTY_SUB_REF = Some( Arc::new(
                    State{cb: AtomicOption::new(), extra: AtomicOption::new(), disposed: AtomicBool::new(true) },
                ));
            });
            SubRef{ state: EMPTY_SUB_REF.as_ref().unwrap().clone() }
        }
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
//impl<'a, F:FnBox()+'static+Send+Sync> IntoSubRef for F
//{
//    #[inline(always)]
//    fn into(self) -> SubRef
//    {
//        SubRef::new(self)
//    }
//}
//impl<'a, F:FnBox()+'static> IntoSubRef for F
//{
//    #[inline(always)]
//    fn into(self) -> SubRef
//    {
//        SubRef::new(self)
//    }
//}
impl IntoSubRef for SubRef
{
    #[inline(always)]
    fn into(self) -> SubRef{ self }
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

        let j2  = thread::spawn(move || { u.unsub(); });

        j.join();
        j2.join();

        assert_eq!(*out3.lock().unwrap(), "AA");
    }

}