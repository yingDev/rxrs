use std::sync::*;
use std::cell::UnsafeCell;

use crate::sync::ReSpinLock;
use crate::*;
use crate::util::{trait_alias::CSS, *};

use self::SubjectState::*;

enum SubjectState<'o, V:Clone, E:Clone, SS:YesNo>
{
    Next(Vec<(Arc<Observer<V,E> + 'o>, Unsub<'o, SS>)>),
    Error(E),
    Complete,
    Drop
}

struct Wrap<'o, V:Clone, E:Clone, SS:YesNo>{ lock: ReSpinLock<SS>, to_drop:UnsafeCell<Vec<*mut SubjectState<'o, V, E, SS>>>, state: UnsafeCell<*mut SubjectState<'o, V, E, SS>> }
unsafe impl <'o, V:CSS, E:CSS> Send for Wrap<'o, V, E, YES> {}
unsafe impl <'o, V:CSS, E:CSS> Sync for Wrap<'o, V, E, YES> {}

pub struct Subject<'o, V:Clone+'o, E:Clone+'o, SS:YesNo>
{
    state: Arc<Wrap<'o,V,E,SS>>,
}
unsafe impl<V:CSS, E:CSS> Send for Subject<'static, V, E, YES> {}
unsafe impl<V:CSS, E:CSS> Sync for Subject<'static, V, E, YES> {}

impl<'o, V:Clone+'o, E:Clone+'o, SS:YesNo> Subject<'o, V, E, SS>
{
    pub fn new() -> Subject<'o, V, E, SS>
    {
        let state_ptr = Box::into_raw(box Next(Vec::new()));
        Subject{ state: Arc::new(Wrap{lock: ReSpinLock::new(), to_drop: UnsafeCell::new(Vec::new()), state: UnsafeCell::new(state_ptr) })  }
    }

    #[inline(never)]
    unsafe fn COMPLETE() -> *mut SubjectState<'o, V, E, SS>
    {
        static mut VAL: *const () = ::std::ptr::null();
        static INIT: Once = ONCE_INIT;
        INIT.call_once(|| VAL = ::std::mem::transmute(Box::into_raw(box (Complete as SubjectState<'o, V, E, SS>))));
        ::std::mem::transmute(VAL)
    }

    #[inline(never)]
    unsafe fn DROP() -> *mut SubjectState<'o, V, E, SS>
    {
        static mut VAL: *const () = ::std::ptr::null();
        static INIT: Once = ONCE_INIT;
        INIT.call_once(|| VAL = ::std::mem::transmute(Box::into_raw(box (Drop as SubjectState<'o, V, E, SS>))));
        ::std::mem::transmute(VAL)
    }

    #[inline(never)]
    fn sub_internal(&self, o: Arc<Observer<V,E> +'o>, make_sub: impl FnOnce()->Unsub<'o, SS>) -> Unsub<'o, SS>
    {
        let Wrap{lock, to_drop, state} = self.state.as_ref();
        let recur = lock.enter();

        let old = unsafe { *state.get() };
        match unsafe { &mut *old } {
            Next(obs) => {
                let sub = make_sub();
                if recur == 0 {
                    obs.push((o, sub.clone()));
                } else {
                    unsafe { (&mut *to_drop.get()).push(old); };

                    let mut vec = Vec::with_capacity(obs.len() + 1);
                    vec.extend( obs.iter().cloned());
                    vec.push((o, sub.clone()));
                    unsafe { *state.get() = Box::into_raw(box Next(vec)); }
                }
                lock.exit();
                return sub;
            },
            Error(e) => o.error(e.clone()),
            Complete => o.complete(),
            Drop => {}
        }
        lock.exit();
        return Unsub::done()
    }

    #[inline(never)]
    fn unsub(state: Weak<Wrap<'o,V,E,SS>>, observer: Weak<Observer<V,E>+'o>)
    {
        if let Some(state) = state.upgrade() {
            if let Some(observer) = observer.upgrade() {
                let Wrap{lock, to_drop, state} = state.as_ref();
                let recur = lock.enter();

                let old = unsafe { *state.get() };
                if let Next(obs) = unsafe { &mut *old } {
                    if recur == 0 {
                        obs.iter().position(|o| Arc::ptr_eq(&o.0, &observer)).map(|i| obs.remove(i))
                            .expect("the observer is expected to be in the vec");
                    } else {
                        let mut vec = Vec::with_capacity(obs.len() - 1 );
                        vec.extend(obs.iter().filter(|o| ! Arc::ptr_eq(&o.0, &observer)).cloned());
                        unsafe {
                            *state.get() = Box::into_raw(box Next(vec));
                            (&mut *to_drop.get()).push(old);
                        }
                    }
                }
                lock.exit();
            }
        }
    }
}


impl<'o, V:Clone+'o, E:Clone+'o, SS:YesNo> ::std::ops::Drop for Subject<'o,V,E,SS>
{
    #[inline(never)]
    fn drop(&mut self)
    {
        let Wrap{lock, to_drop, state} = self.state.as_ref();

        let old = unsafe { *state.get() };
        if let Next(vec) = unsafe { &mut *old } {
            unsafe { *state.get() = Self::DROP(); }
            for (_, sub) in vec { sub.unsub(); }
            unsafe { Box::from_raw(old); }
        }
    }
}

impl<'o, V:Clone+'o, E:Clone+'o> Observable<'o, V, E> for Subject<'o, V, E, NO>
{
    fn sub(&self, observer: impl Observer<V,E>+'o) -> Unsub<'o, NO>
    {
        let o = Arc::new(observer);
        let (state, o2) = (self.state.weak(), o.weak());
        self.sub_internal(o, || Unsub::<NO>::with(|| Self::unsub(state, o2)))
    }
}

impl<V:CSS, E:CSS> ObservableSendSync<V, E> for Subject<'static, V, E, YES>
{
    fn sub(&self, observer: impl Observer<V,E> + Send + Sync+'static) -> Unsub<'static, YES>
    {
        let o = Arc::new(observer);
        let (state, o2) = (self.state.weak(), o.weak());
        self.sub_internal(o, || Unsub::<YES>::with(|| Self::unsub(state, o2)))
    }
}

impl<'o, V:Clone, E:Clone, SS:YesNo> Observer<V,E> for Subject<'o, V,E, SS>
{
    fn next(&self, v: V)
    {
        let Wrap{lock, to_drop, state} = self.state.as_ref();
        let recur = lock.enter();

        let old = unsafe { *state.get() };
        if let Next(vec) = unsafe { &*old } {
            for (o,sub) in vec {
                if ! sub.is_done() { o.next(v.clone()); }
            }

            if recur == 0 {
                let to_drop = unsafe { (&mut *to_drop.get()) };
                if to_drop.len() > 0 {
                    let old = to_drop.clone();
                    to_drop.clear();
                    lock.exit();

                    for ptr in old.into_iter() { unsafe { Box::from_raw(ptr); } }
                    return;
                }
            }
        }

        lock.exit();
    }

    fn error(&self, e: E)
    {
        let Wrap{lock, to_drop, state} = self.state.as_ref();
        let recur = lock.enter();

        let old = unsafe { *state.get() };

        if let Next(vec) = unsafe { &*old } {
            let to_drop = unsafe { (&mut *to_drop.get()) };
            unsafe { *state.get() = Box::into_raw(box Error(e.clone()) ) };
            to_drop.push(old);

            for (o,sub) in vec.iter() {
                if sub.is_done() { continue; }
                sub.unsub();
                o.error(e.clone());
            }

            if recur == 0 {
                if to_drop.len() > 0 {
                    let old = to_drop.clone();
                    to_drop.clear();
                    lock.exit();

                    for ptr in old.into_iter() { unsafe { Box::from_raw(ptr); } }
                    return;
                }
            }
        }

        lock.exit();
    }

    fn complete(&self)
    {
        let Wrap{lock, to_drop, state} = self.state.as_ref();
        let recur = lock.enter();

        let old = unsafe { *state.get() };

        if let Next(vec) = unsafe { &*old } {
            let to_drop = unsafe { (&mut *to_drop.get()) };
            unsafe { *state.get() = Self::COMPLETE(); }
            to_drop.push(old);

            for (o, sub) in vec.iter() {
                if sub.is_done() { continue; }
                sub.unsub();
                o.complete();
            }
            if recur == 0 {
                if to_drop.len() > 0 {
                    let old = to_drop.clone();
                    to_drop.clear();
                    lock.exit();

                    for ptr in old.into_iter() { unsafe { Box::from_raw(ptr); } }
                    return;
                }
            }
        }

        lock.exit();
    }
}

#[cfg(test)]
mod tests
{
    //use test::Bencher;
    use crate::*;
    use std::cell::Cell;
    use std::rc::Rc;
    use std::sync::Arc;
    use std::sync::atomic::*;
    use crate::util::Clones;

    #[test]
    fn smoke()
    {
        let n = Cell::new(0);
        let s = Subject::<i32, (), NO>::new();

        s.sub(|v| { n.replace(v); });

        s.next(1);
        assert_eq!(n.get(), 1);

        s.next(2);
        assert_eq!(n.get(), 2);

        s.complete();
    }

    #[test]
    fn next_after_complete()
    {
        let s = Subject::<i32, (), NO>::new();
        s.sub(|v| assert!(false, "shouldn't call"));

        s.complete();
        s.next(1);
    }

    #[test]
    fn unsub()
    {
        let s = Subject::<i32, (), NO>::new();
        let sub = s.sub(|_| assert!(false, "shouldn't call"));
        sub.unsub();

        s.next(1);
    }

    #[test]
    fn unsub_in_next()
    {
        let (sub, sub2) = Unsub::new().clones();
        let s = Subject::<i32, (), NO>::new();
        s.sub(move |_| sub.unsub());
        sub2.add(s.sub(move |_| assert!(false, "should not happen")));

        s.next(1);
    }

    #[test]
    fn should_complete_only_once()
    {
        let n = Arc::new(AtomicI32::new(0));
        let s = Arc::new(Subject::<i32, (), YES>::new());

        let nn = n.clone();
        s.sub(((), (), move ||{ nn.fetch_add(1, Ordering::SeqCst); }));

        let mut threads = vec![];
        for i in 0..8 {
            let ss = s.clone();
            threads.push(::std::thread::spawn(move ||{

                for j in 0..10 {
                    ss.complete();
                }

            }));
        }

        for t in threads { t.join(); }

        assert_eq!(n.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn drop_should_unsub()
    {
        let n = Rc::new(Cell::new(0));
        let s = Subject::<i32,(), NO>::new();

        for i in 0..10{
            let nn = n.clone();
            s.sub(|v|{}).add(Unsub::<NO>::with(move || { nn.replace(nn.get() + 1); }));
        }

        //s.complete();
        drop(s);

        assert_eq!(n.get(), 10);
    }

    #[test]
    fn deep_recurse()
    {
        let (n,n1) = Rc::new(Cell::new(0)).clones();
        let (s1, s2, s3) = Rc::new(Subject::<i32,(),NO>::new()).clones();

        let sub1 = s1.sub(ObsDropFx { cb: box move ||{ s3.complete(); } });
        let sub2 = s1.sub(|v|{});
        let sub3 = s1.sub(move |v| { sub1.unsub(); sub2.unsub(); });

        s2.next(0);

        struct ObsDropFx{ cb: Box<Fn()> }

        impl Observer<i32, ()> for ObsDropFx
        {
            fn next(&self, v: i32){}
            fn error(&self, e: ()){}
            fn complete(&self){}
        }

        impl Drop for ObsDropFx
        {
            fn drop(&mut self)
            {
                (self.cb)();
            }
        }

    }

}