//taken from https://github.com/crossbeam-rs/crossbeam/blob/master/src/sync/arc_cell.rs

use std::marker::PhantomData;
use std::mem;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

/// A type providing atomic storage and retrieval of an `Arc<T>`.
#[derive(Debug)]
pub struct ArcCell<T>
{
    arc: AtomicUsize,
    PhantomData: PhantomData<Arc<T>>
}

impl<T> Drop for ArcCell<T> {
    #[inline(always)]
    fn drop(&mut self) {
        self.take();
    }
}

impl<T> ArcCell<T> {
    /// Creates a new `ArcCell`.
    pub fn new(t: Arc<T>) -> ArcCell<T> {
        ArcCell { arc: AtomicUsize::new(unsafe { mem::transmute(t) }), PhantomData }
    }

    fn take(&self) -> Arc<T> {
        loop {
            match self.arc.swap(0, Ordering::Acquire) {
                0 => {}
                n => return unsafe { mem::transmute(n) },
            }
        }
    }

    fn put(&self, t: Arc<T>) {
        debug_assert_eq!(self.arc.load(Ordering::SeqCst), 0);
        self.arc
            .store(unsafe { mem::transmute(t) }, Ordering::Release);
    }

    pub fn compare_swap(&self, current: Arc<T>, new: Arc<T>) -> Arc<T>
    {
        unsafe {
            let current = mem::transmute(current);
            let new = mem::transmute(new);
            if self.arc.compare_and_swap(current, new, Ordering::AcqRel) == new {
                let current: Arc<T> = mem::transmute(current);
                mem::transmute(new)
            } else {
                let _old: Arc<T> = mem::transmute(current);
                mem::transmute(current)
            }
        }
    }

    /// Stores a new value in the `ArcCell`, returning the previous
    /// value.
    pub fn set(&self, t: Arc<T>) -> Arc<T> {
        let old = self.take();
        self.put(t);
        old
    }

    /// Returns a copy of the value stored by the `ArcCell`.
    pub fn get(&self) -> Arc<T> {
        let t = self.take();
        // NB: correctness here depends on Arc's clone impl not panicking
        let out = t.clone();
        self.put(t);
        out
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

    use super::*;

    #[test]
    fn basic() {
        let r = ArcCell::new(Arc::new(0));
        assert_eq!(*r.get(), 0);
        assert_eq!(*r.set(Arc::new(1)), 0);
        assert_eq!(*r.get(), 1);
    }

    #[test]
    fn drop_runs() {
        static DROPS: AtomicUsize = ATOMIC_USIZE_INIT;

        struct Foo;

        impl Drop for Foo {
            fn drop(&mut self) {
                DROPS.fetch_add(1, Ordering::SeqCst);
            }
        }

        let r = ArcCell::new(Arc::new(Foo));
        let _f = r.get();
        r.get();
        r.set(Arc::new(Foo));
        drop(_f);
        assert_eq!(DROPS.load(Ordering::SeqCst), 1);
        drop(r);
        assert_eq!(DROPS.load(Ordering::SeqCst), 2);
    }
}