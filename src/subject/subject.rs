use std::sync::*;
use std::cell::UnsafeCell;

use crate::*;
use crate::sync::ReSpinLock;
use crate::util::{alias::{SSs}};

use self::SubjectState::*;

enum SubjectState<'o, SS:YesNo, V, E:Clone>
{
    Next(Vec<(Arc<ActNext<'o, SS, Ref<V>>>, UnsafeCell<Option<Box<ActEcBox<'o, SS, Ref<E>>>>>, Unsub<'o, SS>)>),
    Error(E),
    Complete,
    Drop
}

struct Wrap<'o, SS:YesNo, V, E:Clone>
{
    lock: ReSpinLock<SS>,
    to_drop:UnsafeCell<Vec<*mut SubjectState<'o, SS, V, E>>>,
    state: UnsafeCell<*mut SubjectState<'o, SS, V, E>>
}

unsafe impl <'o, V:SSs, E:SSs+Clone> Send for Wrap<'o, YES, V, E> {}
unsafe impl <'o, V:SSs, E:SSs+Clone> Sync for Wrap<'o, YES, V, E> {}

pub struct Subject<'o, SS:YesNo, V, E:Clone=()>
{
    state: Arc<Wrap<'o,SS,V,E>>,
}
unsafe impl<V:SSs, E:SSs+Clone> Send for Subject<'static, YES, V, E> {}
unsafe impl<V:SSs, E:SSs+Clone> Sync for Subject<'static, YES, V, E> {}

impl<'o, V, E:Clone, SS:YesNo> Subject<'o, SS, V, E>
{
    pub fn new() -> Subject<'o, SS, V, E>
    {
        let state_ptr = Box::into_raw(box Next(Vec::new()));
        Subject{ state: Arc::new(Wrap{lock: ReSpinLock::new(), to_drop: UnsafeCell::new(Vec::new()), state: UnsafeCell::new(state_ptr) })  }
    }

    pub fn new_dyn() -> Box<Self>
    {
        box Subject::new()
    }

    unsafe fn COMPLETE() -> *mut SubjectState<'o, SS, V, E>
    {
        static mut VAL: *const u8 = ::std::ptr::null();
        static INIT: Once = ONCE_INIT;
        INIT.call_once(|| VAL = ::std::mem::transmute(Box::leak(box (Complete as SubjectState<'o, SS, V, E>))));
        ::std::mem::transmute(VAL)
    }

    unsafe fn DROP() -> *mut SubjectState<'o, SS, V, E>
    {
        static mut VAL: *const u8 = ::std::ptr::null();
        static INIT: Once = ONCE_INIT;
        INIT.call_once(|| VAL = ::std::mem::transmute(Box::leak(box (Drop as SubjectState<'o, SS, V, E>))));
        ::std::mem::transmute(VAL)
    }

    #[inline(never)]
    fn sub_internal(&self, next: Arc<ActNext<'o,SS, Ref<V>>>, ec: Box<ActEcBox<'o, SS, Ref<E>>>, make_sub: impl FnOnce()->Unsub<'o, SS>) -> Unsub<'o, SS>
    {
        let Wrap{lock, to_drop, state} = self.state.as_ref();
        let recur = lock.enter();

        match unsafe { &mut **state.get() } {
            Next(obs) => {
                let sub = make_sub();
                if recur == 0 {
                    obs.push((next, UnsafeCell::new(Some(ec)), sub.clone()));
                } else {
                    let mut vec = Vec::with_capacity(obs.len() + 1);
                    vec.extend( obs.iter().map(|(n, ec, sub)| (n.clone(), UnsafeCell::new(unsafe{ &mut *ec.get() }.take()), sub.clone())));
                    vec.push((next, UnsafeCell::new(Some(ec)), sub.clone()));
                    unsafe { Self::change_state(to_drop, state, Box::into_raw(box Next(vec))); }
                }
                lock.exit();
                return sub;
            },
            Error(e) => ec.call_box(Some(By::r(e))),
            Complete => ec.call_box(None),
            Drop => {}
        }
        lock.exit();
        return Unsub::done()
    }

    #[inline(never)]
    fn unsub(state: Weak<Wrap<'o,SS,V,E>>, observer: Weak<ActNext<'o, SS, Ref<V>>>)
    {
        if let Some(state) = state.upgrade() {
            if let Some(observer) = observer.upgrade() {
                let Wrap{lock, to_drop, state} = state.as_ref();
                let recur = lock.enter();

                if let Next(obs) = unsafe { &mut **state.get() } {
                    if recur == 0 {
                        obs.iter().position(|(n,_,_)| Arc::ptr_eq(n, &observer)).map(|i| obs.remove(i))
                            .expect("the observer is expected to be in the vec");
                    } else {
                        let mut vec = Vec::with_capacity(obs.len() - 1 );
                        vec.extend(obs.iter()
                            .filter(|(n,_,_)| ! Arc::ptr_eq(n, &observer))
                            .map(|(n, ec, sub)| (n.clone(), UnsafeCell::new(unsafe{ &mut *ec.get() }.take()), sub.clone() ))
                        );
                        unsafe { Self::change_state(to_drop, state, Box::into_raw(box Next(vec))); }
                    }
                }
                lock.exit();
            }
        }
    }

    unsafe fn change_state(to_drop: &UnsafeCell<Vec<*mut SubjectState<'o, SS, V, E>>>, state: &UnsafeCell<*mut SubjectState<'o, SS, V, E>>, new: *mut SubjectState<'o, SS, V, E> )
    {
        let old = *state.get();
        *state.get() = new;
        (&mut *to_drop.get()).push(old);
    }
}


impl<'s, 'o, V, E:Clone, SS:YesNo> ::std::ops::Drop for Subject<'o,SS,V,E>
{
    fn drop(&mut self)
    {
        let Wrap{lock, to_drop, state} = self.state.as_ref();
        let _ = lock.enter();

        if let Next(vec) = unsafe { &**state.get() } {
            unsafe { Self::change_state(to_drop, state, Self::DROP()); }
            for (_, _, sub) in vec { sub.unsub(); }
        }

        lock.exit();
    }
}

impl<'o, V:'o, E:Clone+'o> Observable<'o, NO, Ref<V>, Ref<E>> for Subject<'o, NO, V, E>
{
    fn sub_dyn(&self, next: Box<ActNext<'o, NO, Ref<V>>>, ec: Box<ActEcBox<'o, NO, Ref<E>>>) -> Unsub<'o, NO>
    {
        let next: Arc<ActNext<'o, NO, Ref<V>>>= next.into();
        let (state, weak_next) = (Arc::downgrade(&self.state), Arc::downgrade(&next));
        self.sub_internal(next, ec, move || Unsub::<NO>::with(move |()| Self::unsub(state, weak_next)))
    }
}

impl<V:Send+Sync+'static, E:Send+Sync+Clone+'static> Observable<'static, YES, Ref<V>, Ref<E>> for Subject<'static, YES, V, E>
{
    fn sub_dyn(&self, next: Box<ActNext<'static, YES, Ref<V>>>, ec: Box<ActEcBox<'static, YES, Ref<E>>>) -> Unsub<'static, YES>
    {
        let next: Arc<ActNext<'static, YES, Ref<V>>+Send+Sync> = sendsync_next_box(next).into();
        let (state, weak_next) = (Arc::downgrade(&self.state), Arc::downgrade(&next));
        self.sub_internal(next, ec, move || Unsub::<YES>::with(move |()| Self::unsub(state, weak_next)))
    }
}


#[inline(never)]
fn drop_garbage<'o, V:'o, E:Clone+'o, SS:YesNo>(to_drop: &UnsafeCell<Vec<*mut SubjectState<'o, SS, V, E>>>, lock:&ReSpinLock<SS>)
{
    let to_drop = unsafe { (&mut *to_drop.get()) };
    if to_drop.len() > 0 {
        let old = to_drop.clone();
        to_drop.clear();
        lock.exit();

        for ptr in old.into_iter() { unsafe { Box::from_raw(ptr); } }
    } else { lock.exit(); }
}

impl<'o, V:'o, E:Clone+'o, SS:YesNo> Subject<'o, SS, V,E>
{
    pub fn next(&self, v: V)
    {
        self.next_ref(&v);
    }

    pub fn ec(&self, e: Option<E>)
    {
        if e.is_some() {
            self.error(e.unwrap());
        } else {
            self.complete();
        }
    }

    pub fn next_ref(&self, v: &V)
    {
        let Wrap{lock, to_drop, state} = self.state.as_ref();
        let recur = lock.enter();

        if let Next(vec) = unsafe { &**state.get() } {
            for (n,_,sub) in vec {
                if ! sub.is_done() { n.call(By::r(v),); }
            }

            if recur == 0 {
                drop_garbage(to_drop, lock);
                return;
            }
        }

        lock.exit();
    }

    pub fn error(&self, e: E)
    {
        let Wrap{lock, to_drop, state} = self.state.as_ref();
        let recur = lock.enter();

        if let Next(vec) = unsafe { &**state.get() } {
            unsafe { Self::change_state(to_drop, state, Box::into_raw(box Error(e.clone())) ) };

            for (_,ec,sub) in vec.iter() {
                if sub.is_done() { continue; }
                sub.unsub();
                unsafe{ &mut *ec.get()}.take().map(|ec| ec.call_box(Some(By::r(&e))));
            }

            if recur == 0 {
                drop_garbage(to_drop, lock);
                return;
            }
        }

        lock.exit();
    }

    pub fn complete(&self)
    {
        let Wrap{lock, to_drop, state} = self.state.as_ref();
        let recur = lock.enter();

        let old = unsafe { *state.get() };

        if let Next(vec) = unsafe { &*old } {
            unsafe { Self::change_state(to_drop, state, Self::COMPLETE()); }

            for (_, ec, sub) in vec.iter() {
                if sub.is_done() { continue; }
                sub.unsub();
                unsafe{ &mut *ec.get() }.take().map(|ec| ec.call_box(None));
            }
            if recur == 0 {
                drop_garbage(to_drop, lock);
                return;
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
        let n = Arc::new(AtomicI32::new(0));
        let s = Arc::new(Subject::<YES, i32>::new());

        let nn = n.clone();
        let ss = s.clone();
        s.sub(move |v:By<_>| { nn.store(*v, Ordering::SeqCst); }, ());

        ::std::thread::spawn(move ||{
            ss.next(123);
        }).join().ok();
        assert_eq!(n.load(Ordering::SeqCst), 123);

        s.next(1);
        assert_eq!(n.load(Ordering::SeqCst), 1);

        s.next(2);
        assert_eq!(n.load(Ordering::SeqCst), 2);

        //expects: `temp` does not live long enough
//        let s = Subject::<NO, i32>::new();
//        let temp = Cell::new(0);
//        s.sub(|v:By<_>| { temp.replace(*v); }, ());



        s.complete();

        let n = Cell::new(0);
        let ss = Subject::<NO, i32>::new_dyn();
        ss.sub_dyn(box |v:By<_>| { n.replace(*v); }, box());
    }

    #[test]
    fn next_after_complete()
    {
        let s = Subject::<NO, i32>::new();
        s.sub(|_:By<_>| assert!(false, "shouldn't call"), ());

        s.complete();
        s.next(1);
    }

    #[test]
    fn unsub()
    {
        let s = Subject::<NO, i32>::new();
        let unsub = s.sub(|_: By<_>| assert!(false, "shouldn't call"), ());
        unsub();

        s.next(1);
    }

    #[test]
    fn unsub_in_next()
    {
        let (sub, sub2) = Unsub::new().clones();
        let s = Subject::<NO, i32>::new();
        s.sub(move |_: By<_>| sub.unsub(), ());
        sub2.add(s.sub(move |_: By<_>| assert!(false, "should not happen"), ()));

        s.next(1);
    }

    #[test]
    fn should_complete_only_once()
    {
//        let n = Arc::new(AtomicI32::new(0));
//        let s = Arc::new(Subject::<i32, (), YES>::new());
//
//        let nn = n.clone();
//        s.sub(((), (), move |()|{ nn.fetch_add(1, Ordering::SeqCst); }));
//
//        let mut threads = vec![];
//        for i in 0..8 {
//            let ss = s.clone();
//            threads.push(::std::thread::spawn(move ||{
//
//                for j in 0..10 {
//                    ss.complete();
//                }
//
//            }));
//        }
//
//        for t in threads { t.join(); }
//
//        assert_eq!(n.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn drop_should_unsub()
    {
        let n = Rc::new(Cell::new(0));
        let s = Subject::<NO, i32>::new();

        for _ in 0..10{
            let nn = n.clone();
            s.sub(|_:By<_>|{}, ()).add(Unsub::<NO>::with(move |()| { nn.replace(nn.get() + 1); }));
        }

        //s.complete();
        drop(s);

        assert_eq!(n.get(), 10);
    }

    #[test]
    fn as_observer()
    {
        let src = Of::value(123);

        let n = std::cell::Cell::new(0);
        let s = Subject::<NO, i32>::new();

        s.sub(|v:By<_>| { n.replace(*v); }, ());

        src.sub(|v:By<_>| s.next(*v), |e: Option<By<_>>| { s.ec(e.map(|e| *e)) } );

        assert_eq!(n.get(), 123);
    }

}