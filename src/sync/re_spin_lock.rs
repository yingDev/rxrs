use std::sync::atomic::*;
use std::cell::UnsafeCell;
use std::marker::PhantomData;
use crate::{YesNo, YES, NO};

/// Recursive SpinLock
pub struct ReSpinLock<SS: YesNo>
{
    recur: UnsafeCell<usize>,
    held: AtomicUsize,
    owner: AtomicUsize,

    recur_nss: UnsafeCell<usize>,

    PhantomData: PhantomData<SS>

}

unsafe impl Send for ReSpinLock<YES>{}
unsafe impl Sync for ReSpinLock<YES>{}

impl<SS: YesNo> ReSpinLock<SS>
{
    pub fn new() -> ReSpinLock<SS>
    {
        ReSpinLock{
            recur: UnsafeCell::new(0), owner: AtomicUsize::new(0), held: AtomicUsize::new(0),
            recur_nss: UnsafeCell::new(0),
            PhantomData
        }
    }

    #[inline(always)]
    pub fn enter(&self) -> usize
    {
        if SS::VALUE {
            let tid = Self::tid();
            loop {
                if self.held.swap(1, Ordering::Acquire) == 0 {
                    debug_assert_eq!( unsafe{ *self.recur.get() }, 0);
                    self.owner.store(tid, Ordering::Relaxed);
                    break;
                } else if self.owner.load(Ordering::Relaxed) == tid {
                    debug_assert!(unsafe { *self.recur.get() } > 0);
                    break;
                }
            }
            unsafe {
                let r = *self.recur.get();
                *self.recur.get() += 1;
                return r;
            }
        } else {
            unsafe {
                let r = *self.recur_nss.get();
                *self.recur_nss.get() += 1;

                println!("enter > {}", r);
                r
            }
        }
    }

    #[inline(always)]
    pub fn exit(&self)
    {
        if SS::VALUE {
            debug_assert_eq!(self.held.load(Ordering::SeqCst), 1);
            debug_assert_eq!(self.owner.load(Ordering::SeqCst), Self::tid());

            unsafe { *self.recur.get() -= 1; }

            if unsafe { *self.recur.get() } == 0 {
                self.owner.store(0, Ordering::Relaxed);
                self.held.store(0, Ordering::Release);
            }
        } else {
            unsafe {
                *self.recur_nss.get() -= 1;
                println!("exit: {}", *self.recur_nss.get());
            }
        }
    }

    fn tid() -> usize
    {
        //from: https://github.com/Amanieu/parking_lot/blob/master/src/remutex.rs

        // The address of a thread-local variable is guaranteed to be unique to the
        // current thread, and is also guaranteed to be recur_nssn-zero.
        thread_local!(static KEY: u8 = unsafe { ::std::mem::uninitialized() });
        KEY.with(|x| x as *const _ as usize)
    }
}

#[cfg(test)]
mod test
{
    #[test]
    fn recur()
    {
        let r = crate::sync::ReSpinLock::<crate::NO>::new();
        assert_eq!(r.enter(), 0);
        a(&r);
        r.exit();

        assert_eq!(r.enter(), 0);
        r.exit();


        fn a(l: &crate::sync::ReSpinLock<crate::NO>)
        {
            assert_eq!(l.enter(), 1);

            b(l);

            l.exit();
        }

        fn b(l: &crate::sync::ReSpinLock<crate::NO>)
        {
            assert_eq!(l.enter(), 2);
            l.exit();
        }
    }
}