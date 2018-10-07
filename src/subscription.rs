use std::sync::atomic::{AtomicBool, Ordering};
use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::boxed::FnBox;

use crate::{YesNo, YES, NO, sync::{ReSpinLock}};

pub struct Subscription<'a, SS:YesNo>
{
    lock: ReSpinLock<SS>,
    done: AtomicBool,
    cbs: UnsafeCell<Option<Vec<Box<FnBox()+'a>>>>,
    PhantomData: PhantomData<SS>
}

unsafe impl Send for Subscription<'static, YES> {}
unsafe impl Sync for Subscription<'static, YES> {}

impl<'a, SS:YesNo> Subscription<'a, SS>
{
    pub fn new() -> Subscription<'a, SS>
    {
        Subscription{ lock: ReSpinLock::new(), done: AtomicBool::new(false), cbs: UnsafeCell::new(None), PhantomData }
    }

    pub fn done() -> Subscription<'a, SS>
    {
        let val = Self::new();
        val.unsubscribe();
        val
    }

    pub fn is_done(&self) -> bool
    {
        self.done.load(Ordering::Relaxed)
    }

    pub fn unsubscribe(&self)
    {
        self.lock.enter();
        self.done.store(true, Ordering::Release);

        unsafe{
            let mut old : &mut Option<_> = &mut *self.cbs.get();
            if let Some(mut vec) = old.take() {
                vec.reverse();
                for cb in vec.drain(..) {
                    cb();
                }
            }

        }
        self.lock.exit();
    }

    fn add_internal(&self, cb: Box<FnBox()+'a>)
    {
        if self.is_done() {
            return cb.call_box(());
        }

        self.lock.enter();

        unsafe {
            let cbs = &mut * self.cbs.get();
            if let Some(mut vec) = cbs.as_mut() {
                vec.push(cb);
            } else {
                let mut vec = Vec::new();
                vec.push(cb);
                *cbs = Some(vec);
            }
        }

        self.lock.exit();
    }

    pub fn add_sendsync(&'static self, cb: impl FnBox() + Send + Sync + 'static) { self.add_internal(Box::new(cb)); }
}

impl<'a> Subscription<'a, NO>
{
    pub fn add(&self, cb: impl FnBox() + 'a)
    {
        self.add_internal(Box::new(cb));
    }
}

impl Subscription<'static, YES>
{
    pub fn add(&self, cb: impl FnBox() + Send + Sync + 'static)
    {
        self.add_internal(Box::new(cb));
    }
}