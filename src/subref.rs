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

static mut EMPTY_SUB_REF: Option<SubRef> = None;
static EMPTY_SUB_REF_INIT: Once = ONCE_INIT;

#[derive(Clone)]
pub struct SubRef
{
    state: Arc<State>,
    _unsub_on_drop: bool,
}

impl Drop for SubRef
{
    fn drop(&mut self)
    {
        if self._unsub_on_drop {
            self.unsub();
        }
    }
}

struct State
{
    disposed: AtomicBool,
    cb: AtomicOption<Box<FnMut()+Send+Sync>>,
    extra: AtomicOption<LinkedList<SubRef>>,
}

impl SubRef
{
    pub fn from_fn<F: FnMut()+'static+Send+Sync>(unsub: F) -> SubRef
    {
         SubRef { state: Arc::new(
            State{
                disposed: AtomicBool::new(false),
                cb: AtomicOption::with(box unsub),
                extra: AtomicOption::with(LinkedList::new())
            }),
            _unsub_on_drop: false,
        }
    }

    pub fn signal() -> SubRef
    {
        SubRef { state: Arc::new(
            State{
                disposed: AtomicBool::new(false),
                cb: AtomicOption::new(),
                extra: AtomicOption::with(LinkedList::new())
            }),
            _unsub_on_drop: false,
        }
    }

    pub fn scoped() -> SubRef
    {
        SubRef { state: Arc::new(
            State{
                disposed: AtomicBool::new(false),
                cb: AtomicOption::new(),
                extra: AtomicOption::with(LinkedList::new())
            }),
            _unsub_on_drop: true,
        }
    }

    pub fn empty() -> SubRef
    {
        unsafe {
            EMPTY_SUB_REF_INIT.call_once(|| {
                //todo
                EMPTY_SUB_REF = Some( SubRef {
                    state: Arc::new(
                    State{cb: AtomicOption::new(), extra: AtomicOption::new(), disposed: AtomicBool::new(true) },
                ),
                    _unsub_on_drop: false,
                } );
            });
            ::std::mem::transmute(EMPTY_SUB_REF.as_ref().unwrap().clone())
        }
    }

    pub fn ptr_eq(&self, other: &SubRef) -> bool
    {
        Arc::ptr_eq(&self.state, &other.state)
    }

    pub fn unsub(&self)
    {
        if self.state.disposed.compare_and_swap(false, true, Ordering::SeqCst){ return; }
        if let Some(mut cb) = self.state.cb.take(Ordering::Acquire) {
            cb();
        }

        loop{
            if let Some(lst) = self.state.extra.take(Ordering::SeqCst){
                lst.iter().for_each(|f| f.unsub());
                break;
            }
        }


    }

    pub fn add<U>(&self, unsub: U) -> &Self where U:IntoSubRef
    {
        let un = unsub.into();
        if Arc::ptr_eq(&un.state, &self.state) {return self;}

        loop {
            if un.disposed() {
                break;
            }else if self.state.disposed.load(Ordering::SeqCst) {
                un.unsub();
                break;
            }else if let Some(mut lst) = self.state.extra.take(Ordering::SeqCst) {
                lst.push_back(un);
                self.state.extra.swap(lst, Ordering::SeqCst);
                break;
            }
        }
        return self;
    }

    pub fn combine<U>(self, other: U) -> Self where U : IntoSubRef
    {
        self.add(other.into());
        self
    }

    #[inline]
    pub fn disposed(&self) -> bool { self.state.disposed.load(Ordering::SeqCst)}
}

pub trait IntoSubRef
{
    fn into(self) -> SubRef;
}
impl<'a, F:Fn()+'static+Send+Sync> IntoSubRef for F
{
    fn into(self) -> SubRef
    {
        SubRef::from_fn(self)
    }
}
impl IntoSubRef for SubRef
{
    fn into(self) -> SubRef{ self }
}
impl IntoSubRef for ()
{
    fn into(self) -> SubRef{ SubRef::empty() }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::thread;
    use observable::*;

    #[test]
    fn disposed()
    {
        let u = SubRef::from_fn(||{});
        u.unsub();
        assert!(u.disposed());
    }

    #[test]
    fn add_after_unsub()
    {
        let u = SubRef::from_fn(||{});
        u.unsub();

        let v = Arc::new(AtomicBool::new(false));
        let v2 = v.clone();
        u.add(move || v2.store(true, Ordering::SeqCst));

        assert!(v.load(Ordering::SeqCst));
    }

    #[test]
    fn threads()
    {
        let out = Arc::new(Mutex::new("".to_owned()));
        let (out2,out3) = (out.clone(), out.clone());

        let u = SubRef::from_fn(||{});
        let (u2,u3) = (u.clone(), u.clone());

        let j = thread::spawn(move || {
            u2.add(move || {  out.lock().unwrap().push_str("A");  });
        });
        u3.add(move || { out2.lock().unwrap().push_str("A"); });

        let j2  = thread::spawn(move || { u.unsub(); });

        j.join();
        j2.join();

        assert_eq!(*out3.lock().unwrap(), "AA");
    }

}