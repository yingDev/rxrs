use crate::*;
use crate::sync::*;
use std::cell::UnsafeCell;
use std::boxed::FnBox;
use std::sync::{Arc, Once, ONCE_INIT};
use std::sync::atomic::{AtomicBool, Ordering};

struct UnsubState<'a, SS:YesNo>
{
    lock: ReSpinLock<SS>,
    done: AtomicBool,
    cb: UnsafeCell<Option<Box<FnBox()+'a>>>,
    cbs: UnsafeCell<Vec<Unsub<'a, SS>>>,
}

impl<'a, SS:YesNo> Drop for UnsubState<'a, SS>
{
    fn drop(&mut self) { self.unsub_then(||{}); }
}

impl<'a, SS:YesNo> UnsubState<'a, SS>
{
    #[inline(always)]
    fn is_done(&self) -> bool
    {
        self.done.load(Ordering::Relaxed)
    }

    fn unsub_then(&self, f: impl FnOnce())
    {
        self.lock.enter();
        if ! self.done.swap(true, Ordering::Release) {

            unsafe{
                if let Some(cb) = (&mut *self.cb.get()).take() {
                    cb.call_box(());
                }
                for cb in (&mut *self.cbs.get()).drain(..) {
                    cb.unsub();
                }
                (&mut *self.cbs.get()).shrink_to_fit();
            }
            self.lock.exit();

            f();
            return;
        }

        self.lock.exit();
    }

    pub fn if_not_done(&self, then: impl FnOnce())
    {
        if self.is_done() { return; }

        self.lock.enter();

        if ! self.is_done() {
            then();
        }

        self.lock.exit();
    }

    #[inline(never)]
    fn add_internal(&self, cb: Unsub<'a, SS>)
    {
        if self.is_done() || cb.is_done() {
            cb.unsub();
            return;
        }

        self.lock.enter();
        unsafe { (&mut * self.cbs.get()).push(cb); }
        self.lock.exit();
    }
}

pub struct Unsub<'a, SS:YesNo>
{
    state: Arc<UnsubState<'a, SS>>
}

impl<'a, SS:YesNo> Clone for Unsub<'a, SS>
{
    fn clone(&self) -> Unsub<'a, SS>
    {
        Unsub { state: self.state.clone() }
    }
}

unsafe impl Send for Unsub<'static, YES> {}
unsafe impl Sync for Unsub<'static, YES> {}

impl<'a, SS:YesNo> Unsub<'a, SS>
{

    pub fn new() -> Unsub<'a, SS>
    {
        Unsub { state: Arc::new(UnsubState { lock: ReSpinLock::new(), done: AtomicBool::new(false), cb: UnsafeCell::new(None), cbs: UnsafeCell::new(Vec::new()) }) }
    }

    #[inline(never)]
    pub fn done() -> Unsub<'a, SS>
    {
        unsafe {
            static mut VAL: *const () = ::std::ptr::null();
            static INIT: Once = ONCE_INIT;
            INIT.call_once(|| VAL = ::std::mem::transmute(Arc::into_raw(Arc::<UnsubState<'a, SS>>::new(UnsubState { lock: ReSpinLock::new(), done: AtomicBool::new(true), cb: UnsafeCell::new(None), cbs: UnsafeCell::new(Vec::new()) }))));
            let arc = Arc::<UnsubState<'a, SS>>::from_raw(::std::mem::transmute(VAL));
            let sub = Unsub { state: arc.clone() };
            ::std::mem::forget(arc);
            return sub;
        }
    }

    #[inline(always)]
    pub fn is_done(&self) -> bool { self.state.is_done() }

    pub fn unsub(&self)
    {
        self.state.unsub_then(||{});
    }

    pub fn unsub_then(&self, f: impl FnOnce())
    {
        self.state.unsub_then(f);
    }

    pub fn if_not_done(&self, then: impl FnOnce())
    {
        self.state.if_not_done(then);
    }

    pub fn add(&self, cb: Unsub<'a, SS>) -> &Self
    {
        self.state.add_internal(cb);
        self
    }

    pub fn added(self, cb: Unsub<'a, SS>) -> Self
    {
        self.add(cb);
        self
    }

    pub fn add_each(&self, b: Unsub<'a, SS>) -> &Self
    {
        self.add(b.added(self.clone()))
    }

    pub fn added_each(self, b: Unsub<'a, SS>) -> Self
    {
        self.add_each(b);
        self
    }
}

impl<'a> Unsub<'a, NO>
{
    pub fn with(cb: impl FnOnce()+'a) -> Unsub<'a, NO>
    {
        Unsub { state: Arc::new(UnsubState { lock: ReSpinLock::new(), done: AtomicBool::new(false), cb:UnsafeCell::new(Some(box cb)), cbs: UnsafeCell::new(Vec::new()) }) }
    }
}

impl Unsub<'static, YES>
{
    pub fn with(cb: impl FnOnce()+Send+Sync+'static) -> Unsub<'static, YES>
    {
        Unsub { state: Arc::new(UnsubState { lock: ReSpinLock::new(), done: AtomicBool::new(false), cb:UnsafeCell::new(Some(box cb)), cbs: UnsafeCell::new(Vec::new()) }) }
    }
}

impl<'o, SS: YesNo> FnOnce<()> for Unsub<'o, SS>
{
    type Output = ();
    extern "rust-call" fn call_once(self, _: ())
    {
        self.unsub();
    }
}

#[cfg(test)]
mod test
{
    use crate::*;
    use std::cell::Cell;
    use std::sync::Arc;

    #[test]
    fn drop_should_unsub()
    {
        let n = Cell::new(0);
        let a = Unsub::<NO>::with(|| { n.replace(1); });

        drop(a);

        assert_eq!(n.get(), 1);
    }

    #[test]
    fn scope()
    {
        let a = Unsub::<NO>::new();
        let r = Arc::new(123);
        {
            //let n = Cell::new(123);
            //a.add(Unsub::<NO>::with(|| println!("n={}", n.get())));
            let _ = Arc::downgrade(&r);

            //let mksub = || Unsub::<NO>::with(|| weak.upgrade());
            //a.add(mksub());
            //a.add(Unsub::<NO>::with(|| weak.upgrade()));

            //let x = weak.upgrade();


        }

        a.unsub();
    }

}