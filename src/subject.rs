use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::Weak;
use std::sync::ONCE_INIT;
use std::sync::Once;
use std::cell::UnsafeCell;
use std::sync::RwLock;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicPtr;

use crate::sync::ArcCell;
use crate::sync::ReSpinLock;
use crate::Observable;
use crate::ObservableSendSync;
use crate::Observer;
use crate::YES;
use crate::NO;
use crate::Subscriber;
use crate::Subscription;
use crate::collection_ext::CollectionExt;
use crate::YesNo;


enum SubjectState<'o, V:Clone, E:Clone, SS>
{
    Next(Vec<Arc<Observer<V,E> + 'o>>),
    Error(E),
    Complete(PhantomData<SS>)
}

unsafe impl <'o, V:Clone, E:Clone> Send for SubjectState<'o, V, E, YES> {}
unsafe impl <'o, V:Clone, E:Clone> Sync for SubjectState<'o, V, E, YES> {}

struct Wrap<'o, V:Clone, E:Clone, SS:YesNo>{ lock: ReSpinLock<SS>, state: UnsafeCell<*mut SubjectState<'o, V, E, SS>> }
unsafe impl <'o, V:Clone, E:Clone> Send for Wrap<'o, V, E, YES> {}
unsafe impl <'o, V:Clone, E:Clone> Sync for Wrap<'o, V, E, YES> {}

pub struct Subject<'o, V:Clone, E:Clone, SS:YesNo>
{
    state: Arc<Wrap<'o,V,E,SS>>,
}

impl<'o, V:Clone, E:Clone, SS:YesNo> Subject<'o, V, E, SS>
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
            INIT.call_once(|| VAL = ::std::mem::transmute(Box::into_raw(box (SubjectState::Complete(PhantomData) as SubjectState<'o, V, E, SS>))));
            ::std::mem::transmute(VAL)
        }
    }

    fn subscribe_internal<'x:'o>(&self, subscriber: Arc<Observer<V,E> +'x>) -> Subscription<'o, SS>
    {
        let Wrap{lock, state} = self.state.as_ref();
        let recur = lock.enter();
        let old = unsafe { *state.get() };

        let sub = match unsafe { &mut *old } {
            SubjectState::Next(obs) => {
                if recur == 0 {
                    obs.push(subscriber);
                } else {
                    let mut vec = Vec::with_capacity(obs.len() + 1);
                    vec.extend( obs.iter().cloned());
                    vec.push(subscriber);
                    unsafe { *state.get() = Box::into_raw(box SubjectState::Next(vec)); }
                }

                Subscription::<SS>::new()
            },
            SubjectState::Error(e) => {
                subscriber.error(e.clone());
                Subscription::<SS>::done()
            },
            SubjectState::Complete(_) => {
                subscriber.complete();
                Subscription::<SS>::done()
            }
        };

        lock.exit();
        sub
    }

    fn unsubscribe(state: Weak<Wrap<'o,V,E,SS>>, subscriber: Arc<Observer<V,E>+'o>)
    {
        if let Some(state) = state.upgrade() {
            let Wrap{lock, state} = state.as_ref();
            let recur = lock.enter();

            if let SubjectState::Next(obs) = unsafe { &mut **state.get() } {

                if let Some(i) = obs.iter().position(|o| Arc::ptr_eq(o, &subscriber)) {
                    if recur == 0 {
                        obs.remove(i);
                    } else {
                        let mut vec = obs.clone();
                        vec.remove(i);
                        unsafe { *state.get() = Box::into_raw(box SubjectState::Next(vec)); }
                    }
                }
            }

            lock.exit();
        }
    }
}

unsafe impl<'o, V:Clone+Send+Sync, E:Clone> Send for Subject<'o, V, E, YES> {}
unsafe impl<'o, V:Clone+Send+Sync, E:Clone> Sync for Subject<'o, V, E, YES> {}

impl<'o, V:Clone, E:Clone, SS:YesNo> Drop for Subject<'o,V,E,SS>
{
    fn drop(&mut self)
    {
        let Wrap{lock, state} = self.state.as_ref();

        let ptr = unsafe { *state.get() };
        if ptr != Self::COMPLETE() {
            unsafe { Box::from_raw(ptr); }
        }
    }
}

impl<'s, 'o, V:Clone+'o, E:Clone+'o> Observable<'s, 'o, V, E> for Subject<'o, V, E, NO>
{
    fn subscribe(&'s self, observer: impl Observer<V,E>+'o) -> Subscription<'o, NO>
    {
        let subscriber = Arc::new(observer);
        let sub = self.subscribe_internal(subscriber.clone());

        if !sub.is_done() {
            let state = Arc::downgrade(&self.state);
            sub.add(move ||{ Self::unsubscribe(state, subscriber) });
        }

        return sub;
    }
}

impl<'s, V:Clone+Send+Sync+'static, E:Clone+Send+Sync+'static> ObservableSendSync<'s, V, E> for Subject<'static, V, E, YES>
{
    fn subscribe(&'s self, observer: impl Observer<V,E> + Send + Sync+'static) -> Subscription<'static, YES>
    {
        let subscriber = Arc::new(observer);
        let sub = self.subscribe_internal(subscriber.clone());

        if !sub.is_done() {
            let state = Arc::downgrade(&self.state);
            sub.add(move ||{ Self::unsubscribe(state, subscriber) });
        }
        return sub;
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
            for o in vec { o.next(v.clone()); }

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

            for o in vec { o.error(e.clone()); }
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

            for o in vec { o.complete(); }
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
    use crate::subject::Subject;
    use crate::Observable;
    use crate::ObservableSendSync;
    use crate::Observer;
    use crate::NO;
    use crate::YES;
    use std::cell::Cell;
    use std::sync::Arc;
    use std::sync::atomic::AtomicI32;
    use std::sync::atomic::Ordering;

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

        sub.unsubscribe();

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

}