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

    #[inline(always)]
    pub fn is_done(&self) -> bool
    {
        self.state.done.load(Ordering::Relaxed)
    }

    pub fn unsub(&self)
    {
        let state = &self.state;
        if ! state.done.swap(true, Ordering::Release) {
            state.lock.enter();

            unsafe{
                if let Some(cb) = (&mut *state.cb.get()).take() {
                    cb();
                }
                for cb in (&mut *state.cbs.get()).drain(..) {
                    cb.unsub();
                }
            }
            state.lock.exit();
        }
    }

    pub fn unsub_then(&self, f: impl Fn())
    {
        let state = &self.state;
        if ! state.done.swap(true, Ordering::Release) {
            state.lock.enter();

            unsafe{
                if let Some(cb) = (&mut *state.cb.get()).take() {
                    cb();
                }
                for cb in (&mut *state.cbs.get()).drain(..) {
                    cb.unsub();
                }
            }
            state.lock.exit();

            f();
        }
    }

    #[inline(never)]
    fn add_internal(&self, cb: Unsub<'a, SS>)
    {
        if self.is_done() {
            cb.unsub();
            return;
        }

        if cb.is_done() {
            return;
        }

        self.state.lock.enter();

        unsafe {
            (&mut * self.state.cbs.get()).push(cb);
        }

        self.state.lock.exit();
    }

    pub fn add(&self, cb: Unsub<'a, SS>)
    {
        self.add_internal(cb);
    }

    pub fn added(self, cb: Unsub<'a, SS>) -> Self
    {
        self.add(cb);
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