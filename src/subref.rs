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

pub struct SubRef<SS:?Sized>
{
    state: Arc<State>,
    PhantomData:PhantomData<*const SS>
}

impl<SS:?Sized> Clone for SubRef<SS>
{
    fn clone(&self) -> SubRef<SS>
    {
        SubRef{ state: self.state.clone(), PhantomData }
    }
}

unsafe impl Send for SubRef<Yes> {}
unsafe impl Sync for SubRef<Yes> {}

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
pub trait NewSubRef<SS: ?Sized, F>
{
    fn new(f:F) -> SubRef<SS>;
}
impl<F: 'static+Send+Sync+FnBox()> NewSubRef<Yes, F> for SubRef<Yes>
{
    fn new(f: F) -> SubRef<Yes>
    {
        SubRef { state: Arc::new(
            State{
                disposed: AtomicBool::new(false),
                cb: AtomicOption::with(box f),
                extra: AtomicOption::with(LinkedList::new())
            }),
            PhantomData
        }
    }
}
impl<F: 'static+FnBox()> NewSubRef<No, F> for SubRef<No>
{
    fn new(f: F) -> SubRef<No>
    {
        let f: Box<FnBox()+'static> = box f;
        let f : Box<FnBox()+'static+Send+Sync> = unsafe{ mem::transmute(f) };

        SubRef { state: Arc::new(
            State{
                disposed: AtomicBool::new(false),
                cb: AtomicOption::with(f),
                extra: AtomicOption::with(LinkedList::new())
            }),
            PhantomData
        }
    }
}

impl SubRef<No>
{
    pub fn add(&self, un: SubRef<No>) -> &Self
    {
        if Arc::ptr_eq(&un.state, &self.state) {return self;}

        loop {
            if un.disposed() {
                break;
            }else if self.state.disposed.load(Ordering::SeqCst) {
                un.unsub();
                break;
            }else if let Some(mut lst) = self.state.extra.take(Ordering::SeqCst) {
                lst.push_back(un.state);
                self.state.extra.swap(lst, Ordering::SeqCst);
                break;
            }
        }
        return self;
    }

    pub fn addf<F>(&self, f: F) where F : FnBox()+'static
    {
        self.add(SubRef::<No>::new(f));
    }
}

impl<SS:?Sized> SubRef<SS>
{
    pub fn signal() -> SubRef<SS>
    {
        SubRef { state: Arc::new(
            State{
                disposed: AtomicBool::new(false),
                cb: AtomicOption::new(),
                extra: AtomicOption::with(LinkedList::new())
            }),
            PhantomData
        }
    }

    #[inline(never)]
    pub fn empty() -> SubRef<SS>
    {
        unsafe {
            EMPTY_SUB_REF_INIT.call_once(|| {
                //todo
                EMPTY_SUB_REF = Some( Arc::new(
                    State{cb: AtomicOption::new(), extra: AtomicOption::new(), disposed: AtomicBool::new(true) },
                ));
            });
            SubRef{ state: EMPTY_SUB_REF.as_ref().unwrap().clone(), PhantomData }
        }
    }

    #[inline(always)]
    pub fn ptr_eq(&self, other: &SubRef<SS>) -> bool
    {
        Arc::ptr_eq(&self.state, &other.state)
    }

    #[inline(never)]
    pub fn unsub(&self)
    {
        self.state.unsub();
    }

    pub fn addf_ss<F>(&self, f: F) where F : FnBox()+Send+Sync+'static
    {
        self.add_ss(SubRef::<Yes>::new(f));
    }

    pub fn add_ss(&self, un: SubRef<Yes>)
    {
        if Arc::ptr_eq(&un.state, &self.state) {return;}

        loop {
            if un.disposed() {
                break;
            }else if self.state.disposed.load(Ordering::SeqCst) {
                un.unsub();
                break;
            }else if let Some(mut lst) = self.state.extra.take(Ordering::SeqCst) {
                lst.push_back(un.state);
                self.state.extra.swap(lst, Ordering::SeqCst);
                break;
            }
        }
    }

    #[inline(always)]
    pub fn disposed(&self) -> bool { self.state.disposed.load(Ordering::SeqCst)}
}

pub trait IntoSubRef<SS:?Sized>
{
    fn into(self) -> SubRef<SS>;
}
impl<SS:?Sized> IntoSubRef<SS> for ()
{
    #[inline(always)]
    fn into(self) -> SubRef<SS> { SubRef::empty() }
}
//impl<'a, F:FnBox()+'static+Send+Sync> IntoSubRef<Yes> for F
//{
//    #[inline(always)]
//    fn into(self) -> SubRef<Yes>
//    {
//        SubRef::new(self)
//    }
//}
//impl<'a, F:FnBox()+'static> IntoSubRef<No> for F
//{
//    #[inline(always)]
//    fn into(self) -> SubRef<No>
//    {
//        SubRef::new(self)
//    }
//}
impl<SS:?Sized> IntoSubRef<SS> for SubRef<SS>
{
    #[inline(always)]
    fn into(self) -> SubRef<SS>{ self }
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
        let a = SubRef::<No>::new(||{});
        a.add_ss(SubRef::<Yes>::new(||{}));
    }

    #[test]
    fn disposed()
    {
        let u = SubRef::<Yes>::new(||{});
        u.unsub();
        assert!(u.disposed());
    }

    #[test]
    fn add_after_unsub()
    {
        let u = SubRef::<Yes>::new(||{});
        u.unsub();

        let v = Arc::new(AtomicBool::new(false));
        let v2 = v.clone();
        u.add_ss(SubRef::new(move || v2.store(true, Ordering::SeqCst)));

        assert!(v.load(Ordering::SeqCst));
    }

    #[test]
    fn threads()
    {
        let out = Arc::new(Mutex::new("".to_owned()));
        let (out2,out3) = (out.clone(), out.clone());

        let u = SubRef::<Yes>::new(||{});
        let (u2,u3) = (u.clone(), u.clone());

        let j = thread::spawn(move || {
            u2.add_ss(SubRef::new(move || {  out.lock().unwrap().push_str("A");  }));
        });
        u3.add_ss(SubRef::new(move || { out2.lock().unwrap().push_str("A"); }));

        let j2  = thread::spawn(move || { u.unsub(); });

        j.join();
        j2.join();

        assert_eq!(*out3.lock().unwrap(), "AA");
    }

}