use std::marker::PhantomData;
use std::sync::*;
use std::sync::atomic::*;
use std::cell::UnsafeCell;

use crate::sync::ReSpinLock;
use crate::*;
use crate::util::trait_alias::CSS;


enum SubjectState<'o, V:Clone, E:Clone, SS:YesNo>
{
    Next(Vec<(Arc<Observer<V,E> + 'o>, Unsub<'o, SS>)>),
    Error(E),
    Complete,
    Drop
}

struct Wrap<'o, V:Clone, E:Clone, SS:YesNo>{ lock: ReSpinLock<SS>, state: UnsafeCell<*mut SubjectState<'o, V, E, SS>> }
unsafe impl <'o, V:Clone, E:Clone> Send for Wrap<'o, V, E, YES> {}
unsafe impl <'o, V:Clone, E:Clone> Sync for Wrap<'o, V, E, YES> {}

pub struct Subject<'o, V:Clone+'o, E:Clone+'o, SS:YesNo>
{
    state: Arc<Wrap<'o,V,E,SS>>,
}

impl<'o, V:Clone+'o, E:Clone+'o, SS:YesNo> Subject<'o, V, E, SS>
{
    pub fn new() -> Subject<'o, V, E, SS>
    {
        let state_ptr = Box::into_raw(box SubjectState::Next(Vec::new()));
        Subject{ state: Arc::new(Wrap{lock: ReSpinLock::new(), state: UnsafeCell::new(state_ptr) })  }
    }

    fn COMPLETE() -> *mut SubjectState<'o, V, E, SS>
    {
        unsafe {
            static mut VAL: *const () = ::std::ptr::null();
            static INIT: Once = ONCE_INIT;
            INIT.call_once(|| VAL = ::std::mem::transmute(Box::into_raw(box (SubjectState::Complete as SubjectState<'o, V, E, SS>))));
            ::std::mem::transmute(VAL)
        }
    }

    fn DROP() -> *mut SubjectState<'o, V, E, SS>
    {
        unsafe {
            static mut VAL: *const () = ::std::ptr::null();
            static INIT: Once = ONCE_INIT;
            INIT.call_once(|| VAL = ::std::mem::transmute(Box::into_raw(box (SubjectState::Drop as SubjectState<'o, V, E, SS>))));
            ::std::mem::transmute(VAL)
        }
    }

    #[inline(never)]
    fn subscribe_internal(&self, observer: Arc<Observer<V,E> +'o>) -> Unsub<'o, SS>
    {
        let Wrap{lock, state} = self.state.as_ref();
        let recur = lock.enter();
        let old = unsafe { *state.get() };

        let sub = match unsafe { &mut *old } {
            SubjectState::Next(obs) => {
                let (weak_state, observer_clone) = (Arc::downgrade(&self.state),observer.clone());
                let sub = Unsub::<SS>::with(move || Self::unsubscribe(weak_state, observer_clone));

                if recur == 0 {
                    obs.push((observer, sub.clone()));
                } else {
                    let mut vec = Vec::with_capacity(obs.len() + 1);
                    vec.extend( obs.iter().cloned());
                    vec.push((observer, sub.clone()));
                    unsafe { *state.get() = Box::into_raw(box SubjectState::Next(vec)); }
                }
                sub
            },
            SubjectState::Error(e) => {
                observer.error(e.clone());
                Unsub::<SS>::done()
            },
            SubjectState::Complete => {
                observer.complete();
                Unsub::<SS>::done()
            },
            SubjectState::Drop => {
                Unsub::<SS>::done()
            }
        };

        lock.exit();
        sub
    }

    #[inline(never)]
    fn unsubscribe(state: Weak<Wrap<'o,V,E,SS>>, observer: Arc<Observer<V,E>+'o>)
    {
        if let Some(state) = state.upgrade() {
            let Wrap{lock, state} = state.as_ref();
            let recur = lock.enter();

            if let SubjectState::Next(obs) = unsafe { &mut **state.get() } {

                if let Some(i) = obs.iter().position(|o| Arc::ptr_eq(&o.0, &observer)) {
                    if recur == 0 {
                        let (o, sub) = obs.remove(i);
                        sub.unsub();
                    } else {
                        let mut vec = obs.clone();
                        let (o, sub) = vec.remove(i);
                        unsafe { *state.get() = Box::into_raw(box SubjectState::Next(vec)); }

                        sub.unsub();
                    }
                }
            }

            lock.exit();
        }
    }
}



unsafe impl<V:CSS, E:Clone> Send for Subject<'static, V, E, YES> {}
unsafe impl<V:CSS, E:Clone> Sync for Subject<'static, V, E, YES> {}

impl<'o, V:Clone+'o, E:Clone+'o, SS:YesNo> Drop for Subject<'o,V,E,SS>
{
    fn drop(&mut self)
    {
        let Wrap{lock, state} = self.state.as_ref();

        lock.enter();

        let old = unsafe { *state.get() };
        if let SubjectState::Next(vec) = unsafe { &*old } {
            unsafe { *state.get() = Self::DROP(); }
            lock.exit();

            for (_, sub) in vec { sub.unsub(); }
            unsafe { Box::from_raw(old); }
            return;
        }

        lock.exit();
    }
}

impl<'o, V:Clone+'o, E:Clone+'o> Observable<'o, V, E> for Subject<'o, V, E, NO>
{
    fn subscribe(&self, observer: impl Observer<V,E>+'o) -> Unsub<'o, NO>
    {
        self.subscribe_internal(Arc::new(observer))
    }
}

impl<V:CSS, E:CSS> ObservableSendSync<V, E> for Subject<'static, V, E, YES>
{
    fn subscribe(&self, observer: impl Observer<V,E> + Send + Sync+'static) -> Unsub<'static, YES>
    {
        self.subscribe_internal(Arc::new(observer))
    }
}

impl<'o, V:Clone, E:Clone, SS:YesNo> Observer<V,E> for Subject<'o, V,E, SS>
{
    fn next(&self, v: V)
    {
        let Wrap{lock, state} = self.state.as_ref();
        let recur = lock.enter();

        let old = unsafe { *state.get() };
        if let SubjectState::Next(vec) = unsafe { &*old } {
            for (o,sub) in vec {
                if sub.is_done() { continue; }
                o.next(v.clone());
            }

            if unsafe { *state.get() != old } && recur == 0 {
                lock.exit();
                unsafe { Box::from_raw(old); }
                return;
            }
        }

        lock.exit();
    }

    fn error(&self, e: E)
    {
        let Wrap{lock, state} = self.state.as_ref();
        let recur = lock.enter();

        let old = unsafe { *state.get() };

        if let SubjectState::Next(vec) = unsafe { &*old } {
            unsafe { *state.get() = Box::into_raw(box SubjectState::Error(e.clone()) ) };
            lock.exit();

            for (o,sub) in vec {
                if sub.is_done() { continue; }
                sub.unsub();
                o.error(e.clone());
            }
            if recur == 0 { unsafe { Box::from_raw(old); } }
            return;
        }

        lock.exit();
    }

    fn complete(&self)
    {
        let Wrap{lock, state} = self.state.as_ref();
        let recur = lock.enter();

        let old = unsafe { *state.get() };

        if let SubjectState::Next(vec) = unsafe { &*old } {
            unsafe { *state.get() = Self::COMPLETE(); }
            lock.exit();

            for (o, sub) in vec {
                if sub.is_done() { continue; }

                sub.unsub();
                o.complete();
            }
            if recur == 0 { unsafe { Box::from_raw(old); } }
            return;
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
    use crate::util::CloneN;

    #[test]
    fn smoke()
    {
        let n = Cell::new(0);
        let s = Subject::<i32, (), NO>::new();

        s.subscribe(|v| { n.replace(v); });

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
        s.subscribe(|v| assert!(false, "shouldn't call"));

        s.complete();
        s.next(1);
    }

    #[test]
    fn unsub()
    {
        let s = Subject::<i32, (), NO>::new();
        let sub = s.subscribe(|_| assert!(false, "shouldn't call"));
        sub.unsub();

        s.next(1);
    }

    #[test]
    fn unsub_in_next()
    {
        let (sub, sub2) = Unsub::new().cloned2();
        let s = Subject::<i32, (), NO>::new();
        s.subscribe(move |_| sub.unsub());
        sub2.add(s.subscribe(move |_| assert!(false, "should not happen")));

        s.next(1);
    }

    #[test]
    fn should_complete_only_once()
    {
        let n = Arc::new(AtomicI32::new(0));
        let s = Arc::new(Subject::<i32, (), YES>::new());

        let nn = n.clone();
        s.subscribe(((), (), move ||{ nn.fetch_add(1, Ordering::SeqCst); }));

        let mut threads = vec![];
        for i in 0..8 {
            let ss = s.clone();
            threads.push(::std::thread::spawn(move ||{
                for j in 0..1000 {
                    ss.complete();
                }
            }));
        }

        for t in threads { t.join(); }

        assert_eq!(n.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn should_release_subs()
    {
        let n = Rc::new(Cell::new(0));
        let s = Subject::<i32,(), NO>::new();

        for i in 0..10{
            let nn = n.clone();
            s.subscribe(|v|{}).add(move || { nn.replace(nn.get() + 1); });
        }

        //s.complete();
        drop(s);

        assert_eq!(n.get(), 10);


    }

}