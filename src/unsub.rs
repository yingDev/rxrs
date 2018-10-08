use std::sync::atomic::{AtomicBool, Ordering};
use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::boxed::FnBox;
use std::sync::{Arc, Once, ONCE_INIT};

use crate::{YesNo, YES, NO, sync::{ReSpinLock}};

struct State<'a, SS:YesNo>
{
    lock: ReSpinLock<SS>,
    done: AtomicBool,
    cb: UnsafeCell<Option<Box<FnBox() + 'a>>>,
    cbs: UnsafeCell<Vec<Unsub<'a, SS>>>,
}

impl<'a, SS:YesNo> Drop for State<'a, SS>
{
    fn drop(&mut self) { self.unsub_then(||{}); }
}

impl<'a, SS:YesNo> State<'a, SS>
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
                    cb();
                }
                for cb in (&mut *self.cbs.get()).drain(..) {
                    cb.unsub();
                }
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
    state: Arc<State<'a, SS>>
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
        Unsub { state: Arc::new(State{ lock: ReSpinLock::new(), done: AtomicBool::new(false), cb: UnsafeCell::new(None), cbs: UnsafeCell::new(Vec::new()) }) }
    }

    pub fn with(cb: impl FnBox() + 'a) -> Unsub<'a, SS>
    {
        Unsub { state: Arc::new(State{ lock: ReSpinLock::new(), done: AtomicBool::new(false), cb: UnsafeCell::new(Some(box cb)), cbs: UnsafeCell::new(Vec::new()) }) }
    }

    #[inline(always)]
    pub fn is_done(&self) -> bool { self.state.is_done() }

    pub fn unsub(&self)
    {
        self.state.unsub_then(||{});
    }

    pub fn unsub_then(&self, f: impl Fn())
    {
        self.state.unsub_then(f);
    }

    pub fn if_not_done(&self, then: impl FnOnce())
    {
        self.state.if_not_done(then);
    }

    #[inline(never)]
    pub fn done() -> Unsub<'a, SS>
    {
        unsafe {
            static mut VAL: *const () = ::std::ptr::null();
            static INIT: Once = ONCE_INIT;
            INIT.call_once(|| VAL = ::std::mem::transmute(Arc::into_raw(Arc::<State<'a, SS>>::new(State{ lock: ReSpinLock::new(), done: AtomicBool::new(true), cb: UnsafeCell::new(None), cbs: UnsafeCell::new(Vec::new()) }))));
            let arc = Arc::<State<'a, SS>>::from_raw(::std::mem::transmute(VAL));
            let sub = Unsub { state: arc.clone() };
            ::std::mem::forget(arc);
            return sub;
        }
    }

    pub fn add(&self, cb: impl IntoUnsub<'a, SS>)
    {
        self.state.add_internal(cb.into_unsub());
    }

    pub fn added(self, cb: impl IntoUnsub<'a, SS>) -> Self
    {
        self.add(cb.into_unsub());
        self
    }

    pub fn add_each(&self, b: &Unsub<'a, SS>)
    {
        self.add(b.clone());
        b.add(self.clone());
    }

    pub fn added_each(self, b: &Unsub<'a, SS>) -> Self
    {
        self.add_each(b);
        self
    }
}

pub trait IntoUnsub<'a, SS:YesNo>
{
    fn into_unsub(self) -> Unsub<'a, SS>;
}

impl<'a, SS:YesNo> IntoUnsub<'a, SS> for Unsub<'a, SS>
{
    fn into_unsub(self) -> Unsub<'a, SS> { self }
}

impl<'a, F:FnBox()+'a> IntoUnsub<'a, NO> for F
{
    fn into_unsub(self) -> Unsub<'a, NO> { Unsub::with(self) }
}

impl<F:FnBox()+Send+Sync+'static> IntoUnsub<'static, YES> for F
{
    fn into_unsub(self) -> Unsub<'static, YES> { Unsub::with(self) }
}

#[cfg(test)]
mod test
{
    use std::cell::Cell;
    use crate::*;

    #[test]
    fn drop_should_unsub()
    {
        let n = Cell::new(0);
        let a = Unsub::<NO>::with(|| { n.replace(1); });

        drop(a);

        assert_eq!(n.get(), 1);
    }
}